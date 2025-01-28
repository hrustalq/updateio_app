export interface Settings {
  steamPath?: string;
  epicPath?: string;
  autoUpdate: boolean;
  updateInterval: number; // в часах
  notifications: boolean;
}

export type SettingsUpdate = Partial<Settings>; 