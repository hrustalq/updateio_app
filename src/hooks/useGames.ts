import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getInstalledGames, refreshGamesList, updateGame, checkGameUpdates } from '@/lib/api';
import type { Game } from '@/types/games';
import { useEffect } from 'react';
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event';

interface UseGamesReturn {
  games: Game[];
  isLoading: boolean;
  refresh: () => Promise<Game[]>;
  isRefreshing: boolean;
  updateGame: (gameId: string) => Promise<void>;
  isUpdating: boolean;
  checkUpdates: (gameId: string) => Promise<boolean>;
  isChecking: boolean;
}

export function useGames(): UseGamesReturn {
  const queryClient = useQueryClient();

  const { data: games = [], isLoading } = useQuery<Game[]>({
    queryKey: ['games'],
    queryFn: getInstalledGames,
  });

  const refreshMutation = useMutation({
    mutationFn: refreshGamesList,
    onSuccess: (newGames) => {
      queryClient.setQueryData(['games'], newGames);
    },
  });

  const updateMutation = useMutation({
    mutationFn: updateGame,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['games'] });
    },
  });

  const checkUpdatesMutation = useMutation({
    mutationFn: checkGameUpdates,
  });

  // Подписываемся на события обновления
  useEffect(() => {
    let unlistenFn: UnlistenFn;

    async function listenToEvents() {
      unlistenFn = await listen<Game>('update-progress', (event: Event<Game>) => {
        // Обновляем состояние игры в кэше
        queryClient.setQueryData(['games'], (oldGames: Game[] | undefined) => {
          if (!oldGames) return oldGames;
          return oldGames.map(game => 
            game.id === event.payload.id ? { ...game, ...event.payload } : game
          );
        });
      });
    }

    listenToEvents();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [queryClient]);

  return {
    games,
    isLoading,
    refresh: refreshMutation.mutateAsync,
    isRefreshing: refreshMutation.isPending,
    updateGame: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
    checkUpdates: checkUpdatesMutation.mutateAsync,
    isChecking: checkUpdatesMutation.isPending,
  };
} 