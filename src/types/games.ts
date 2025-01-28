export interface Game {
  id: string;
  name: string;
  platform: 'steam' | 'epic';
  installPath: string;
  lastUpdate?: string;
  updateStatus?: {
    isUpdating: boolean;
    progress?: number;
    error?: string;
  };
}

export interface UpdateProgress {
  gameId: string;
  progress: number;
  status: 'downloading' | 'installing' | 'complete' | 'error';
  message?: string;
} 