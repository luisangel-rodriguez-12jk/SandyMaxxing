import { NavLink, Outlet } from "react-router-dom";
import { t } from "../i18n/es";
import UserSwitcher from "./UserSwitcher";

const links: { to: string; label: string; icon: string }[] = [
  { to: "/usuarios", label: t.nav.usuarios, icon: "👪" },
  { to: "/mi-plan", label: t.nav.miPlan, icon: "🥗" },
  { to: "/cocinar", label: t.nav.cocinar, icon: "🍳" },
  { to: "/progreso", label: t.nav.progreso, icon: "📈" },
  { to: "/ajustes", label: t.nav.ajustes, icon: "⚙️" },
];

export default function Layout() {
  return (
    <div className="flex h-full">
      <aside className="w-64 shrink-0 bg-gradient-to-b from-mint-600 to-mint-800 text-white flex flex-col">
        <div className="px-5 pt-6 pb-4 border-b border-white/10">
          <div className="text-xl font-semibold tracking-tight">{t.app}</div>
          <div className="text-xs text-mint-100/80">{t.tagline}</div>
        </div>
        <nav className="flex-1 overflow-y-auto p-3 space-y-1">
          {links.map((l) => (
            <NavLink
              key={l.to}
              to={l.to}
              className={({ isActive }) =>
                "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition " +
                (isActive
                  ? "bg-white/20 text-white font-medium"
                  : "text-mint-50/90 hover:bg-white/10")
              }
            >
              <span className="text-base leading-none">{l.icon}</span>
              <span>{l.label}</span>
            </NavLink>
          ))}
        </nav>
        <div className="p-3 border-t border-white/10">
          <UserSwitcher />
        </div>
      </aside>
      <main className="flex-1 overflow-y-auto p-8">
        <Outlet />
      </main>
    </div>
  );
}
