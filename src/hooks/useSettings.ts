import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getSettings, updateSettings, selectDirectory } from '@/lib/api';
import type { Settings, SettingsUpdate } from '@/types/settings';

interface UseSettingsReturn {
  settings: Settings | undefined;
  isLoading: boolean;
  updateSettings: (update: SettingsUpdate) => Promise<void>;
  isUpdating: boolean;
  selectDirectory: () => Promise<string>;
  isSelecting: boolean;
}

export function useSettings(): UseSettingsReturn {
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery<Settings>({
    queryKey: ['settings'],
    queryFn: getSettings,
  });

  const updateMutation = useMutation({
    mutationFn: updateSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
    },
  });

  const selectDirectoryMutation = useMutation({
    mutationFn: selectDirectory,
  });

  return {
    settings,
    isLoading,
    updateSettings: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
    selectDirectory: () => selectDirectoryMutation.mutateAsync(),
    isSelecting: selectDirectoryMutation.isPending,
  };
} 