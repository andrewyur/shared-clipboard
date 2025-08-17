import { redirect } from '@sveltejs/kit';
import { platform } from '@tauri-apps/plugin-os';
import { checkInputMonitoringPermission } from 'tauri-plugin-macos-permissions-api';

export async function load() {
  if (platform() !== "macos") {
    throw redirect(302, '/history');
  }

  const permissions = await Promise.all([ checkInputMonitoringPermission()]) 
  if( permissions.every(p => p) ) {
    throw redirect(302, '/history');
  } else {
    throw redirect(302, '/permissions');
  }
}