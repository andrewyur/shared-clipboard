use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::contents::{load_pinned, store_pinned, Contents};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

const HISTORY_LEN: usize = 20;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(serde::Serialize)]
struct ManagedItem {
    contents: Contents,
    is_pinned: bool, // whether to expose pin or unpin button
    id: u32,         // items with the same id have the same contents
}

impl ManagedItem {
    fn try_from_clipboard(app: &AppHandle) -> Option<Self> {
        Contents::try_from_clipboard(app).map(|contents| Self {
            contents,
            is_pinned: false,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        })
    }
    fn from_contents(contents: Contents, is_pinned: bool) -> Self {
        Self {
            contents,
            is_pinned,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}
pub struct Manager {
    store: HashMap<u32, ManagedItem>,
    history: VecDeque<u32>,
    pinned: Vec<u32>,
    ignore: bool,
    app: AppHandle,
}

impl Manager {
    pub fn new(app: AppHandle) -> Self {
        log::info!("created clipboard manager");
        let mut store = HashMap::new();
        let mut history = VecDeque::with_capacity(HISTORY_LEN);

        if let Some(current_item) = ManagedItem::try_from_clipboard(&app) {
            history.push_front(current_item.id);
            store.insert(current_item.id, current_item);
        }

        let pinned = load_pinned(&app)
            .unwrap_or_else(|e| {
                log::error!("Unable to load pinned items: {:#}", e);
                vec![]
            })
            .into_iter()
            .map(|c| {
                let item = ManagedItem::from_contents(c, true);
                let item_id = item.id;
                store.insert(item_id, item);
                item_id
            })
            .collect();

        Self {
            store,
            history,
            pinned,
            ignore: false,
            app,
        }
    }

    pub fn emit(&self) {
        let hydrated_history: Vec<&ManagedItem> = self
            .history
            .iter()
            .map(|id| self.store.get(id).unwrap())
            .collect();
        let hydrated_pinned: Vec<&ManagedItem> = self
            .pinned
            .iter()
            .map(|id| self.store.get(id).unwrap())
            .collect();

        let _ = self
            .app
            .emit(
                "update",
                json!({
                    "history": hydrated_history,
                    "pinned": hydrated_pinned
                }),
            )
            .map_err(|e| log::error!("Could not emit pinned event {}", e));
    }

    // when this is called, we already know the current clipboard contents are outdated
    pub fn check(&mut self) {
        if self.ignore {
            self.ignore = false
        } else {
            if let Some(new_item) = ManagedItem::try_from_clipboard(&self.app) {
                if self.history.len() == HISTORY_LEN {
                    let popped = self.history.pop_back().unwrap();
                    if !self.store.get(&popped).unwrap().is_pinned {
                        self.store.remove(&popped);
                    }
                }

                self.history.push_front(new_item.id);
                self.store.insert(new_item.id, new_item);
            }
            self.emit();
        }
    }

    pub fn copy(&mut self, id: u32) {
        if let Some((index, _)) = self
            .history
            .iter()
            .enumerate()
            .find(|(_, f_id)| id == **f_id)
        {
            if index == 0 {
                return; // don't copy an item currently in the clipboard
            }
            self.history.remove(index);
            self.history.push_front(id);
        } else {
            if self.history.len() == HISTORY_LEN {
                let popped = self.history.pop_back().unwrap();
                if !self.store.get(&popped).unwrap().is_pinned {
                    self.store.remove(&popped);
                }
            }
            self.history.push_front(id);
        }

        self.ignore = true;
        self.store
            .get(&id)
            .unwrap()
            .contents
            .try_to_clipboard(&self.app);
        self.emit();
    }

    pub fn pin(&mut self, id: u32) {
        if !self.pinned.contains(&id) {
            self.pinned.insert(0, id);
            self.store.get_mut(&id).unwrap().is_pinned = true;
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
            .find(|(_, f_id)| id == **f_id)
        {
            self.pinned.remove(index);

            if self.history.contains(&id) {
                self.store.get_mut(&id).unwrap().is_pinned = false;
            } else {
                self.store.remove(&id);
            }
            self.emit();
            self.save_pinned();
        } else {
            log::warn!("tried to unpin an item that wasnt in the pinned items vec");
            log::warn!("{}, {:?}", id, self.pinned);
        }
    }

    fn save_pinned(&self) {
        let pinned_contents = self
            .store
            .values()
            .filter_map(|f| if f.is_pinned { Some(&f.contents) } else { None })
            .collect::<Vec<_>>();
        if let Err(e) = store_pinned(&pinned_contents, &self.app) {
            log::error!("Could not store pinned items: {:#}", e)
        }
    }
}
