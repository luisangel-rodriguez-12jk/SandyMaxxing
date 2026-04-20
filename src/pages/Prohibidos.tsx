import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";
import { api, Food } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";

export default function Prohibidos() {
  const qc = useQueryClient();
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const { data: foods } = useQuery({
    queryKey: ["foods", activeUserId],
    queryFn: () => api.foodsList(activeUserId),
    enabled: activeUserId != null,
  });

  const toggle = useMutation({
    mutationFn: (v: { food_id: number; forbidden: boolean }) =>
      api.forbiddenSet(activeUserId!, v.food_id, v.forbidden),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["foods"] }),
  });

  // Agrupamos preservando el orden en que vienen del backend (ya ordenado por folleto).
  const byGroup = useMemo(() => {
    const order: string[] = [];
    const map = new Map<string, Food[]>();
    (foods ?? []).forEach((f) => {
      if (!map.has(f.group_name)) {
        map.set(f.group_name, []);
        order.push(f.group_name);
      }
      map.get(f.group_name)!.push(f);
    });
    return order.map((g) => [g, map.get(g)!] as const);
  }, [foods]);

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">
          {t.common.seleccionarUsuario} para marcar sus alimentos prohibidos.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-mint-700">{t.prohibidos.subtitulo}</p>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {byGroup.map(([groupName, list]) => (
          <div key={groupName} className="card">
            <h3 className="font-semibold text-mint-700 mb-3">{groupName}</h3>
            <div className="flex flex-wrap gap-1.5">
              {list.map((f) => (
                <button
                  key={f.id}
                  onClick={() =>
                    toggle.mutate({ food_id: f.id, forbidden: !f.forbidden })
                  }
                  title={`${f.portion_quantity} ${f.portion_unit}`}
                  className={
                    f.forbidden
                      ? "rounded-full px-3 py-1 text-xs font-medium bg-red-500 text-white shadow-sm"
                      : "rounded-full px-3 py-1 text-xs font-medium bg-mint-100 text-mint-800 hover:bg-mint-200"
                  }
                >
                  {f.name}
                </button>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
