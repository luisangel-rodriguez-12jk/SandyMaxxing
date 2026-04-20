import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api, FoodGroup } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";

const MEALS: { key: string; label: string }[] = [
  { key: "desayuno", label: t.meals.desayuno },
  { key: "colacion1", label: t.meals.colacion1 },
  { key: "comida", label: t.meals.comida },
  { key: "colacion2", label: t.meals.colacion2 },
  { key: "cena", label: t.meals.cena },
];

function todayISO(): string {
  return new Date().toISOString().slice(0, 10);
}

type Draft = Record<string, Record<number, number>>; // meal → groupId → portions

export default function PlanSemanal() {
  const qc = useQueryClient();
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const [weekStart, setWeekStart] = useState(todayISO());
  const [draft, setDraft] = useState<Draft>({});
  const [dirty, setDirty] = useState(false);
  const [message, setMessage] = useState<{ kind: "ok" | "err"; text: string } | null>(null);

  const { data: groups } = useQuery({ queryKey: ["groups"], queryFn: api.foodGroupsList });
  const { data: diet } = useQuery({
    queryKey: ["diet", activeUserId, weekStart],
    queryFn: () => api.dietGet(activeUserId!, weekStart),
    enabled: activeUserId != null,
  });

  // Cuando llega la dieta (o cambia el usuario/semana) cargamos el draft con los valores guardados.
  useEffect(() => {
    if (!diet) return;
    const d: Draft = {};
    for (const p of diet.portions) {
      d[p.meal_type] = d[p.meal_type] ?? {};
      d[p.meal_type][p.group_id] = p.portions;
    }
    setDraft(d);
    setDirty(false);
  }, [diet?.id]);

  const save = useMutation({
    mutationFn: async () => {
      if (!diet) throw new Error("No hay dieta activa para esta semana");
      const dietId = diet.id;
      for (const m of MEALS) {
        for (const g of groups ?? []) {
          const newVal = draft[m.key]?.[g.id] ?? 0;
          const oldVal =
            diet.portions.find((p) => p.meal_type === m.key && p.group_id === g.id)?.portions ?? 0;
          if (newVal !== oldVal) {
            await api.dietSetPortion({
              diet_id: dietId,
              meal_type: m.key,
              group_id: g.id,
              portions: newVal,
            });
          }
        }
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["diet"] });
      setDirty(false);
      setMessage({ kind: "ok", text: "✓ Porciones guardadas correctamente" });
      setTimeout(() => setMessage(null), 2500);
    },
    onError: (e: any) => setMessage({ kind: "err", text: String(e) }),
  });

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">
          {t.common.seleccionarUsuario} para registrar sus porciones.
        </p>
      </div>
    );
  }

  const setCell = (meal: string, groupId: number, v: number) => {
    setDraft((d) => ({ ...d, [meal]: { ...(d[meal] ?? {}), [groupId]: v } }));
    setDirty(true);
  };

  const orderedGroups: FoodGroup[] = groups ?? [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between flex-wrap gap-3">
        <label className="flex items-center gap-2 text-sm">
          <span className="label">{t.dietPlanner.semana}</span>
          <input
            type="date"
            className="input"
            value={weekStart}
            onChange={(e) => setWeekStart(e.target.value)}
          />
        </label>
        <div className="flex items-center gap-3">
          {message && (
            <span
              className={
                "text-sm " +
                (message.kind === "ok" ? "text-green-700" : "text-red-600")
              }
            >
              {message.text}
            </span>
          )}
          <button
            className="btn-primary"
            disabled={!dirty || save.isPending}
            onClick={() => save.mutate()}
          >
            {save.isPending ? t.common.cargando : t.common.guardar}
          </button>
        </div>
      </div>

      <div className="card overflow-x-auto">
        <table className="table min-w-[720px]">
          <thead>
            <tr>
              <th>{t.common.grupo}</th>
              {MEALS.map((m) => (
                <th key={m.key}>{m.label}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {orderedGroups.map((g) => (
              <tr key={g.id}>
                <td className="font-medium">{g.name}</td>
                {MEALS.map((m) => (
                  <td key={m.key}>
                    <input
                      type="number"
                      step="0.5"
                      min={0}
                      className="input w-24"
                      value={draft[m.key]?.[g.id] ?? 0}
                      onChange={(e) => setCell(m.key, g.id, Number(e.target.value || 0))}
                    />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {dirty && (
        <p className="text-xs text-amber-700">
          Tienes cambios sin guardar — presiona <strong>{t.common.guardar}</strong> para registrarlos.
        </p>
      )}
    </div>
  );
}
