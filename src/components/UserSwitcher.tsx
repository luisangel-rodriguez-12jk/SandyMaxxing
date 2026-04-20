import { useQuery } from "@tanstack/react-query";
import { api } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";
import { useEffect } from "react";

export default function UserSwitcher() {
  const { data: users } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const { activeUserId, setActiveUserId } = useActiveUser();

  useEffect(() => {
    if (!activeUserId && users && users.length > 0) {
      setActiveUserId(users[0].id);
    }
  }, [users, activeUserId, setActiveUserId]);

  return (
    <div>
      <div className="text-[10px] uppercase tracking-wider text-mint-100/70 mb-1">
        Usuario activo
      </div>
      <select
        value={activeUserId ?? ""}
        onChange={(e) =>
          setActiveUserId(e.target.value ? Number(e.target.value) : null)
        }
        className="w-full rounded-lg bg-white/10 border border-white/20 text-white text-sm px-2 py-1.5
                   focus:outline-none focus:bg-white/20"
      >
        {/* Placeholder sólo visible mientras no hay un usuario activo.
            Con `hidden` el <select> no lo incluye al desplegar opciones. */}
        <option value="" disabled hidden>
          {t.common.seleccionarUsuario}
        </option>
        {users?.map((u) => (
          <option key={u.id} value={u.id} className="text-mint-900">
            {u.name}
          </option>
        ))}
      </select>
    </div>
  );
}
