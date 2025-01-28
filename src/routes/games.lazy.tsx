import { createLazyFileRoute } from '@tanstack/react-router'
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { RefreshCw, Download, Check, AlertCircle } from "lucide-react"
import { useGames } from '@/hooks/useGames'
import { useEffect } from 'react'
import { listenToUpdateProgress } from '@/lib/api'
import type { UpdateProgress } from '@/types/games'

export const Route = createLazyFileRoute('/games')({
  component: Games,
})

function Games() {
  const { 
    games, 
    isLoading, 
    refresh, 
    isRefreshing,
    updateGame,
    isUpdating,
    checkUpdates,
    isChecking
  } = useGames();

  useEffect(() => {
    const handleProgress = (progress: UpdateProgress) => {
      // В реальном приложении здесь можно обновлять состояние через zustand
      console.log('Update progress:', progress);
    };

    listenToUpdateProgress(handleProgress);
  }, []);

  if (isLoading) {
    return <div>Загрузка...</div>;
  }

  return (
    <div className="grid gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Установленные игры</h1>
        <Button 
          variant="outline" 
          size="icon"
          onClick={() => refresh()}
          disabled={isRefreshing}
        >
          <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
        </Button>
      </div>

      <div className="grid gap-4">
        {games.map((game) => (
          <Card key={game.installPath}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle>{game.name}</CardTitle>
                  <CardDescription>{game.platform}</CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button 
                    variant="outline" 
                    size="icon"
                    onClick={async () => {
                      const needsUpdate = await checkUpdates(game.id);
                      if (needsUpdate) {
                        await updateGame(game.id);
                      }
                    }}
                    disabled={isChecking || isUpdating || game.updateStatus?.isUpdating}
                  >
                    {isChecking ? (
                      <RefreshCw className="h-4 w-4 animate-spin" />
                    ) : game.updateStatus?.isUpdating ? (
                      <Download className="h-4 w-4 animate-pulse" />
                    ) : (
                      <Check className="h-4 w-4" />
                    )}
                  </Button>
                  {game.updateStatus?.error && (
                    <AlertCircle className="h-4 w-4 text-destructive" />
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-sm text-muted-foreground mb-2">
                Последнее обновление: {game.lastUpdate || 'Никогда'}
              </div>
              {game.updateStatus?.isUpdating && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Загрузка обновления</span>
                    <span className="text-sm text-muted-foreground">
                      {game.updateStatus.progress}%
                    </span>
                  </div>
                  <Progress value={game.updateStatus.progress} />
                </div>
              )}
              {game.updateStatus?.error && (
                <div className="text-sm text-destructive mt-2">
                  {game.updateStatus.error}
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
} 