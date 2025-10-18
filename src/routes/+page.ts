import { redirect } from '@sveltejs/kit';
import { platform } from '@tauri-apps/plugin-os';
import { checkAccessibilityPermission, checkInputMonitoringPermission } from 'tauri-plugin-macos-permissions-api';
// import { info } from '@tauri-apps/plugin-log';

export async function load() {
  if (platform() !== "macos") {
    throw redirect(302, '/history');
  }

  const permissions = await Promise.all([ checkInputMonitoringPermission(), checkAccessibilityPermission()]) 
  // info(permissions.toString())
  if( permissions.every(p => p) ) {
    throw redirect(302, '/history');
  } else {
    throw redirect(302, '/permissions');
  }
}