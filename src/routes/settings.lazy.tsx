import { createLazyFileRoute } from '@tanstack/react-router'
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useSettings } from '@/hooks/useSettings'

export const Route = createLazyFileRoute('/settings')({
  component: Settings,
})

function Settings() {
  const { 
    settings, 
    isLoading, 
    updateSettings,
    selectDirectory,
    isSelecting,
    isUpdating
  } = useSettings();

  if (isLoading) {
    return <div>Загрузка...</div>;
  }

  return (
    <div className="grid gap-4">
      <h1 className="text-3xl font-bold">Настройки</h1>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Общие настройки</CardTitle>
            <CardDescription>
              Основные настройки приложения
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between space-x-2">
              <Label htmlFor="auto-update">Автоматическое обновление</Label>
              <Switch 
                id="auto-update"
                checked={settings?.autoUpdate}
                onCheckedChange={(checked) => 
                  updateSettings({ key: 'autoUpdate', value: checked })
                }
                disabled={isUpdating}
              />
            </div>
            <div className="flex items-center justify-between space-x-2">
              <Label htmlFor="notifications">Уведомления</Label>
              <Switch 
                id="notifications"
                checked={settings?.notifications}
                onCheckedChange={(checked) => 
                  updateSettings({ key: 'notifications', value: checked })
                }
                disabled={isUpdating}
              />
            </div>
            <div className="space-y-2">
              <Label>Интервал проверки обновлений</Label>
              <Select 
                value={String(settings?.checkInterval)}
                onValueChange={(value) => 
                  updateSettings({ key: 'checkInterval', value: Number(value) })
                }
                disabled={isUpdating}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="15">15 минут</SelectItem>
                  <SelectItem value="30">30 минут</SelectItem>
                  <SelectItem value="60">1 час</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Пути к играм</CardTitle>
            <CardDescription>
              Настройка путей к библиотекам игр
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Steam</Label>
              <div className="flex space-x-2">
                <Input 
                  placeholder="Путь к Steam" 
                  value={settings?.paths.steam || ''}
                  readOnly
                  disabled={isUpdating}
                />
                <Button 
                  variant="secondary"
                  onClick={async () => {
                    try {
                      const path = await selectDirectory();
                      await updateSettings({
                        key: 'paths',
                        value: { ...settings?.paths, steam: path }
                      });
                    } catch (e) {
                      // Пользователь отменил выбор
                    }
                  }}
                  disabled={isUpdating || isSelecting}
                >
                  {isUpdating ? "Сохранение..." : "Обзор"}
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <Label>Epic Games</Label>
              <div className="flex space-x-2">
                <Input 
                  placeholder="Путь к Epic Games"
                  value={settings?.paths.epic || ''}
                  readOnly
                  disabled={isUpdating}
                />
                <Button 
                  variant="secondary"
                  onClick={async () => {
                    try {
                      const path = await selectDirectory();
                      await updateSettings({
                        key: 'paths',
                        value: { ...settings?.paths, epic: path }
                      });
                    } catch (e) {
                      // Пользователь отменил выбор
                    }
                  }}
                  disabled={isUpdating || isSelecting}
                >
                  {isUpdating ? "Сохранение..." : "Обзор"}
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
} 