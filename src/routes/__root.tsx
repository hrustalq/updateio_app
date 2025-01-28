import { createRootRoute, Link, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'
import { Button } from "@/components/ui/button"
import { 
  LayoutDashboard, 
  Settings, 
  GamepadIcon,
} from "lucide-react"

export const Route = createRootRoute({
  component: () => (
    <div className="min-h-screen bg-background">
      {/* Sidebar */}
      <aside className="fixed left-0 top-0 z-40 h-screen w-64 border-r">
        <div className="flex h-full flex-col">
          {/* Logo */}
          <div className="border-b px-6 py-4">
            <h2 className="text-lg font-semibold">Update.io</h2>
          </div>
          
          {/* Navigation */}
          <nav className="flex-1 space-y-1 px-3 py-4">
            <Link to="/">
              {({ isActive }) => (
                <Button
                  variant={isActive ? "secondary" : "ghost"}
                  className="w-full justify-start gap-2"
                >
                  <LayoutDashboard size={20} />
                  Дашборд
                </Button>
              )}
            </Link>
            <Link to="/games">
              {({ isActive }) => (
                <Button
                  variant={isActive ? "secondary" : "ghost"}
                  className="w-full justify-start gap-2"
                >
                  <GamepadIcon size={20} />
                  Игры
                </Button>
              )}
            </Link>
            <Link to="/settings">
              {({ isActive }) => (
                <Button
                  variant={isActive ? "secondary" : "ghost"}
                  className="w-full justify-start gap-2"
                >
                  <Settings size={20} />
                  Настройки
                </Button>
              )}
            </Link>
          </nav>
        </div>
      </aside>

      {/* Main Content */}
      <main className="pl-64">
        <div className="container p-6">
          <Outlet />
        </div>
      </main>

      {process.env.NODE_ENV === 'development' && <TanStackRouterDevtools />}
    </div>
  ),
})