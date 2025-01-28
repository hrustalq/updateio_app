import { invoke } from '@tauri-apps/api/core';
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event';
import type { Game, UpdateProgress } from '@/types/games';
import type { Settings, SettingsUpdate } from '@/types/settings';

// Games API
export async function getInstalledGames(): Promise<Game[]> {
  return invoke<Game[]>('get_installed_games');
}

export async function checkGameUpdates(gameId: string): Promise<boolean> {
  return invoke<boolean>('check_game_updates', { gameId });
}

export async function updateGame(gameId: string): Promise<void> {
  return invoke<void>('update_game', { gameId });
}

export async function refreshGamesList(): Promise<Game[]> {
  return invoke<Game[]>('refresh_games_list');
}

// Settings API
export async function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export async function updateSettings(update: SettingsUpdate): Promise<void> {
  return invoke('update_settings', { update });
}

export async function selectDirectory(): Promise<string> {
  return invoke('select_directory');
}

// Events
export function listenToUpdateProgress(callback: (progress: UpdateProgress) => void): Promise<UnlistenFn> {
  return listen<UpdateProgress>('update-progress', (event: Event<UpdateProgress>) => {
    callback(event.payload);
  });
} 