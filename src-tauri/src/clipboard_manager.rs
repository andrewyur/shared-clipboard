use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::contents::{load_pinned, store_pinned, Contents};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

const HISTORY_LEN: usize = 20;

struct ContentsStore {
    store: HashSet<Arc<Contents>>
}

impl ContentsStore {
    fn new() -> Self {
        Self { store: HashSet::new() }
    }

    fn add(&mut self, item: Contents) -> &Arc<Contents> {
        let key = Arc::new(item);
        self.store.insert(key.clone());
        self.store.get(&key).unwrap()
    }

    fn prune(&mut self) {
        self.store.retain(| rc | Arc::strong_count(rc) > 1);
    }

    fn get_by_id(&self, id: u32) -> Option<&Arc<Contents>>{
        self.store.iter().find(|c| c.id() == id)
    }
}

pub struct ClipboardManager {
    store: ContentsStore,
    history: VecDeque<Arc<Contents>>,
    pinned: Vec<Arc<Contents>>,
    app: AppHandle,
}

impl ClipboardManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        log::info!("created clipboard manager");

        let app = app_handle.clone();

        let mut store = ContentsStore::new();
        let mut history = VecDeque::with_capacity(HISTORY_LEN);

        let pinned = load_pinned(&app)
            .unwrap_or_else(|e| {
                log::error!("Unable to load pinned items: {:#}", e);
                vec![]
            })
            .into_iter()
            .map(|item| {
                Arc::clone(store.add(item))
            })
            .collect();

        if let Some(item) = Contents::try_from_clipboard(&app) {
            history.push_front(Arc::clone(store.add(item)));
        }

        Self {
            store,
            history,
            pinned,
            app,
        }
    }

    pub fn emit(&self) {
        let _ = self
            .app
            .emit(
                "update",
                json!({
                    "history": self.history,
                    "pinned": self.pinned
                }),
            )
            .map_err(|e| log::error!("Could not emit pinned event {}", e));
    }

    // when this is called, we already know the current clipboard contents are outdated
    pub fn check(&mut self) {
        if let Some(new_item) = Contents::try_from_clipboard(&self.app) {
            if self.history.len() == HISTORY_LEN {
                self.history.pop_back().unwrap();
                self.store.prune();
            }
    
            self.history.push_front(Arc::clone(self.store.add(new_item)));
        }
        self.emit();
    }

    pub fn copy(&mut self, id: u32) {
        if let Some((index, _)) = self
            .history
            .iter()
            .enumerate()
            .find(|(_, c)| id == c.id())
        {
            if index == 0 {
                return; // don't copy an item currently in the clipboard
            }
            self.history.remove(index).unwrap();
        } 

        self.store.get_by_id(id).map(|c| c.try_to_clipboard(&self.app));
    }

    pub fn pin(&mut self, id: u32) {
        let Some(item) = self.store.get_by_id(id) else {
            return
        };

        if !self.pinned.contains(item) {
            self.pinned.insert(0, Arc::clone(item));
            self.emit();
            self.save_pinned();
        } else {
            log::warn!("tried to pin an item that was already in the pinned items vec");
            log::warn!("{}, {:?}", id, self.pinned);
        }
    }

    pub fn unpin(&mut self, id: u32) {
        if let Some((index, _)) = self
            .pinned
            .iter()
            .enumerate()
            .find(|(_, f)| id == f.id())
        {
            self.pinned.remove(index);
            self.store.prune();
            self.emit();
            self.save_pinned();
        } else {
            log::warn!("tried to unpin an item that wasnt in the pinned items vec");
            log::warn!("{}, {:?}", id, self.pinned);
        }
    }

    fn save_pinned(&self) {
        if let Err(e) = store_pinned(&self.pinned, &self.app) {
            log::error!("Could not store pinned items: {:#}", e)
        }
    }
}
