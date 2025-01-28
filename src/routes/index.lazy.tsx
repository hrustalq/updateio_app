import { createLazyFileRoute } from '@tanstack/react-router'
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"

export const Route = createLazyFileRoute('/')({
  component: Index,
})

function Index() {
  return (
    <div className="grid gap-4">
      <h1 className="text-3xl font-bold">Дашборд</h1>
      
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Активные обновления</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm">Counter-Strike 2</span>
                  <span className="text-sm text-muted-foreground">45%</span>
                </div>
                <Progress value={45} />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Статистика</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Всего игр:</span>
                <span className="font-medium">12</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Обновлено сегодня:</span>
                <span className="font-medium">3</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}