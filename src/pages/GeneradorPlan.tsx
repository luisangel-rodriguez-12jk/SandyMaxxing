import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { api, PlanMeal, PlanResult, SavedPlan } from "../api/invoke";
import { t } from "../i18n/es";

function todayISO(): string {
  return new Date().toISOString().slice(0, 10);
}

function addDaysISO(iso: string, days: number): string {
  const d = new Date(iso + "T00:00:00");
  d.setDate(d.getDate() + days);
  return d.toISOString().slice(0, 10);
}

function daysBetween(a: string, b: string): number {
  const da = new Date(a + "T00:00:00").getTime();
  const db = new Date(b + "T00:00:00").getTime();
  return Math.round((db - da) / 86_400_000);
}

export default function GeneradorPlan() {
  const qc = useQueryClient();
  const { data: users } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const { data: hasKey } = useQuery({
    queryKey: ["has_key"],
    queryFn: api.settingsHasKey,
  });
  const { data: savedPlans } = useQuery({
    queryKey: ["saved_plans"],
    queryFn: api.savedPlansList,
  });

  const [selected, setSelected] = useState<number[]>([]);
  const [startDate, setStartDate] = useState(todayISO());
  const [endDate, setEndDate] = useState(addDaysISO(todayISO(), 6));
  const [notes, setNotes] = useState("");
  const [plan, setPlan] = useState<PlanResult | null>(null);
  const [currentPlanId, setCurrentPlanId] = useState<number | null>(null);
  const [planName, setPlanName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const [smoothieWarningDismissed, setSmoothieWarningDismissed] = useState(false);

  // Edición de una comida específica
  const [editing, setEditing] = useState<{ day: string; mealIdx: number } | null>(null);
  const [editInstruction, setEditInstruction] = useState("");

  // Consulta los licuados de todos los usuarios seleccionados para saber si
  // mostrar la advertencia de "no hay licuados".
  const smoothieQueries = useQueries({
    queries: selected.map((uid) => ({
      queryKey: ["smoothies", uid],
      queryFn: () => api.smoothiesList(uid),
      staleTime: 30_000,
    })),
  });
  const totalSmoothies = useMemo(
    () =>
      smoothieQueries.reduce(
        (sum, q) => sum + (q.data?.length ?? 0),
        0,
      ),
    [smoothieQueries],
  );
  const hasAnyUserLoaded = smoothieQueries.length > 0 && smoothieQueries.every((q) => q.data);
  const showSmoothieWarning =
    selected.length > 0 &&
    hasAnyUserLoaded &&
    totalSmoothies === 0 &&
    !smoothieWarningDismissed;

  const generate = useMutation({
    mutationFn: () => api.planGenerate(selected, startDate, endDate, notes || null),
    onSuccess: (p) => {
      setPlan(p);
      setCurrentPlanId(null);
      setError(null);
      if (!planName) setPlanName(`Plan ${startDate}`);
    },
    onError: (e: any) => setError(String(e)),
  });

  const tweak = useMutation({
    mutationFn: (v: { day: string; meal: PlanMeal; instruction: string }) =>
      api.planTweakMeal(selected, startDate, v.day, v.meal, v.instruction),
    onSuccess: (newMeal, vars) => {
      if (!plan) return;
      const updated: PlanResult = {
        days: plan.days.map((d) => {
          if (d.day !== vars.day) return d;
          const meals = d.meals.map((m, i) =>
            editing && editing.day === vars.day && editing.mealIdx === i ? newMeal : m,
          );
          return { ...d, meals };
        }),
      };
      setPlan(updated);
      setEditing(null);
      setEditInstruction("");
    },
    onError: (e: any) => setError(String(e)),
  });

  const savePlan = useMutation({
    mutationFn: () => {
      if (!plan) throw new Error("No hay plan para guardar");
      if (!planName.trim()) throw new Error("Dale un nombre al plan antes de guardar");
      return api.savedPlansUpsert(
        currentPlanId,
        planName.trim(),
        startDate,
        selected,
        plan,
        notes || null,
      );
    },
    onSuccess: (id) => {
      setCurrentPlanId(id);
      setMessage("✓ Plan guardado");
      setTimeout(() => setMessage(null), 2500);
      qc.invalidateQueries({ queryKey: ["saved_plans"] });
    },
    onError: (e: any) => setError(String(e)),
  });

  const deletePlan = useMutation({
    mutationFn: (id: number) => api.savedPlansDelete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["saved_plans"] });
      if (currentPlanId != null) {
        setPlan(null);
        setCurrentPlanId(null);
      }
    },
  });

  function loadPlan(sp: SavedPlan) {
    try {
      const p: PlanResult = JSON.parse(sp.plan_json);
      const uids: number[] = JSON.parse(sp.user_ids_json);
      setPlan(p);
      setCurrentPlanId(sp.id);
      setPlanName(sp.name);
      setStartDate(sp.week_start);
      // Derivamos endDate de la longitud del plan guardado (1 día por entry).
      const nDays = Math.max(1, p.days.length);
      setEndDate(addDaysISO(sp.week_start, nDays - 1));
      setSelected(uids);
      setNotes(sp.notes ?? "");
      setError(null);
    } catch (e: any) {
      setError("No se pudo cargar el plan: " + String(e));
    }
  }

  async function exportPdf() {
    if (!plan) return;
    try {
      const bytes = await api.pdfPlan(plan, planName || `Plan ${startDate}`);
      const path = await saveDialog({
        filters: [{ name: "PDF", extensions: ["pdf"] }],
        defaultPath: `${(planName || "plan").replace(/[\\/:*?"<>|]/g, "_")}.pdf`,
      });
      if (path) {
        await writeFile(path, new Uint8Array(bytes));
        setMessage("✓ PDF exportado");
        setTimeout(() => setMessage(null), 2500);
      }
    } catch (e: any) {
      setError(String(e));
    }
  }

  function onGenerateClick() {
    setError(null);
    if (selected.length === 0) {
      setError("Selecciona al menos un usuario para generar el plan.");
      return;
    }
    if (!hasKey) {
      setError("Falta configurar la clave de OpenAI. Ve a Ajustes → Clave IA.");
      return;
    }
    if (!startDate || !endDate) {
      setError("Selecciona las fechas de inicio y fin del plan.");
      return;
    }
    const diff = daysBetween(startDate, endDate);
    if (diff < 0) {
      setError("La fecha de fin debe ser igual o posterior a la de inicio.");
      return;
    }
    if (diff > 30) {
      setError("El rango no puede exceder 31 días. Parte el plan en bloques más cortos.");
      return;
    }
    generate.mutate();
  }

  function startNew() {
    setPlan(null);
    setCurrentPlanId(null);
    setPlanName("");
    setError(null);
  }

  const nDays = Math.max(1, daysBetween(startDate, endDate) + 1);

  return (
    <div className="space-y-6 max-w-5xl pb-24">
      {/* Panel de control */}
      <div className="card space-y-4">
        <div>
          <span className="label">Usuarios del plan</span>
          <div className="flex flex-wrap gap-2 mt-1">
            {users && users.length > 0 ? (
              users.map((u) => {
                const on = selected.includes(u.id);
                return (
                  <button
                    key={u.id}
                    onClick={() =>
                      setSelected((s) =>
                        on ? s.filter((i) => i !== u.id) : [...s, u.id],
                      )
                    }
                    className={
                      on
                        ? "rounded-full px-3 py-1 text-xs font-medium bg-mint-600 text-white"
                        : "rounded-full px-3 py-1 text-xs font-medium bg-mint-100 text-mint-800 hover:bg-mint-200"
                    }
                  >
                    {u.name}
                  </button>
                );
              })
            ) : (
              <span className="text-xs text-mint-700">
                Aún no hay usuarios. Crea al menos uno en la sección Usuarios.
              </span>
            )}
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
          <label className="space-y-1">
            <span className="label">Inicio</span>
            <input
              type="date"
              className="input"
              value={startDate}
              onChange={(e) => {
                const v = e.target.value;
                setStartDate(v);
                // Si se rompe el rango, empuja el fin.
                if (v && endDate && new Date(v) > new Date(endDate)) {
                  setEndDate(addDaysISO(v, 6));
                }
              }}
            />
          </label>
          <label className="space-y-1">
            <span className="label">Fin</span>
            <input
              type="date"
              className="input"
              value={endDate}
              min={startDate}
              onChange={(e) => setEndDate(e.target.value)}
            />
          </label>
          <label className="space-y-1 md:col-span-2">
            <span className="label">{t.plan.notas}</span>
            <input
              className="input"
              placeholder="Ej. evitar picante, bajo sodio, etc."
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </label>
        </div>
        <p className="text-xs text-mint-700">
          Se generarán <strong>{nDays}</strong> día{nDays === 1 ? "" : "s"} · 5 comidas por día.
        </p>

        {!hasKey && (
          <p className="text-xs text-amber-700 bg-amber-50 p-2 rounded">
            ⚠ Necesitas configurar la clave de OpenAI en Ajustes → Clave IA para generar planes.
          </p>
        )}

        {showSmoothieWarning && (
          <div className="flex items-start gap-2 text-xs text-amber-800 bg-amber-50 border border-amber-200 p-2 rounded">
            <span className="mt-0.5">⚠</span>
            <div className="flex-1">
              <p>
                <strong>Sin licuados registrados.</strong> El plan se generará sin incorporar
                licuados en las colaciones. Si quieres incluirlos, ve a{" "}
                <em>Mi plan → Licuados</em> y registra al menos uno para cada persona.
              </p>
            </div>
            <button
              className="text-amber-800 hover:text-amber-950 font-medium"
              onClick={() => setSmoothieWarningDismissed(true)}
            >
              Omitir
            </button>
          </div>
        )}

        {error && <p className="text-sm text-red-600">{error}</p>}
        {message && <p className="text-sm text-green-700">{message}</p>}

        <div className="flex justify-end gap-2 flex-wrap">
          {plan && (
            <button className="btn-ghost" onClick={startNew}>
              Nuevo plan
            </button>
          )}
          <button
            className="btn-primary"
            disabled={generate.isPending}
            onClick={onGenerateClick}
          >
            {generate.isPending ? t.common.cargando : t.plan.generarSemana}
          </button>
        </div>
      </div>

      {/* Plan actual */}
      {plan && (
        <>
          <div className="card space-y-3">
            <label className="space-y-1 block">
              <span className="label">Nombre del plan</span>
              <input
                className="input"
                placeholder="p. ej. Semana del 20 de abril"
                value={planName}
                onChange={(e) => setPlanName(e.target.value)}
              />
            </label>
          </div>

          <div className="space-y-4">
            {plan.days.map((day) => (
              <div key={day.day} className="card">
                <h3 className="font-semibold text-mint-700 mb-3 text-lg">{day.day}</h3>
                <div className="space-y-4">
                  {day.meals.map((m, i) => (
                    <div key={i} className="border-l-2 border-mint-300 pl-3">
                      <div className="flex items-baseline gap-2 flex-wrap">
                        <span className="chip">{m.meal_type}</span>
                        <span className="font-medium">{m.name}</span>
                        <button
                          className="ml-auto text-xs text-mint-700 hover:text-mint-900 underline"
                          onClick={() => {
                            setEditing({ day: day.day, mealIdx: i });
                            setEditInstruction("");
                          }}
                        >
                          ✎ editar con IA
                        </button>
                      </div>
                      {editing?.day === day.day && editing.mealIdx === i && (
                        <div className="mt-2 space-y-2 p-3 bg-mint-50 rounded">
                          <input
                            className="input"
                            placeholder="Ej. 'cámbialo por algo con pollo y arroz'"
                            value={editInstruction}
                            onChange={(e) => setEditInstruction(e.target.value)}
                          />
                          <div className="flex gap-2 justify-end">
                            <button
                              className="btn-ghost"
                              onClick={() => {
                                setEditing(null);
                                setEditInstruction("");
                              }}
                            >
                              {t.common.cancelar}
                            </button>
                            <button
                              className="btn-primary"
                              disabled={!editInstruction.trim() || tweak.isPending}
                              onClick={() =>
                                tweak.mutate({
                                  day: day.day,
                                  meal: m,
                                  instruction: editInstruction,
                                })
                              }
                            >
                              {tweak.isPending ? t.common.cargando : "Aplicar cambio"}
                            </button>
                          </div>
                        </div>
                      )}
                      <div className="text-xs text-mint-800 mt-2 whitespace-pre-wrap">
                        <strong>Preparación:</strong> {m.instructions}
                      </div>
                      <ul className="text-xs text-mint-800 mt-2 list-disc ml-5">
                        {m.ingredients.map((ing, j) => (
                          <li key={j}>
                            {ing.quantity} {ing.unit} · {ing.name}
                          </li>
                        ))}
                      </ul>
                      {m.per_user_portions.length > 0 && (
                        <div className="mt-2 text-xs text-mint-700 bg-mint-50 p-2 rounded space-y-1.5">
                          {m.per_user_portions.map((p, k) => (
                            <div key={k}>
                              <strong>{p.user}:</strong> {p.notes}
                              {p.portions_consumed && p.portions_consumed.length > 0 && (
                                <div className="flex flex-wrap gap-1 mt-1">
                                  {p.portions_consumed.map((gp, j) => (
                                    <span
                                      key={j}
                                      className="inline-flex items-center rounded-full bg-white border border-mint-200 text-mint-800 px-2 py-0.5 text-[10px]"
                                    >
                                      {gp.group}: {gp.portions}
                                    </span>
                                  ))}
                                </div>
                              )}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </>
      )}

      {/* Planes guardados */}
      {savedPlans && savedPlans.length > 0 && (
        <div className="card">
          <h3 className="font-semibold text-mint-700 mb-3">{t.tabs.guardados}</h3>
          <ul className="space-y-2">
            {savedPlans.map((sp) => (
              <li
                key={sp.id}
                className="flex items-center justify-between border-b border-mint-100 pb-2 last:border-0"
              >
                <div>
                  <div className="font-medium text-sm">{sp.name}</div>
                  <div className="text-xs text-mint-700">
                    Inicio: {sp.week_start} · Creado: {sp.created_at}
                    {currentPlanId === sp.id && (
                      <span className="ml-2 chip">actual</span>
                    )}
                  </div>
                </div>
                <div className="flex gap-2">
                  <button className="btn-ghost" onClick={() => loadPlan(sp)}>
                    Abrir
                  </button>
                  <button
                    className="btn-danger"
                    onClick={() => {
                      if (confirm(`¿Eliminar "${sp.name}"?`)) {
                        deletePlan.mutate(sp.id);
                      }
                    }}
                  >
                    ×
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {!plan && (!savedPlans || savedPlans.length === 0) && (
        <p className="text-sm text-mint-700">{t.plan.sinPlan}</p>
      )}

      {/* Barra flotante con Guardar / Exportar — aparece cuando hay plan. */}
      {plan && (
        <div className="fixed bottom-0 left-0 right-0 md:left-64 z-40 border-t border-mint-200 bg-white/95 backdrop-blur px-4 py-3 shadow-lg">
          <div className="max-w-5xl mx-auto flex items-center gap-2 flex-wrap">
            <span className="text-xs text-mint-700 truncate flex-1 min-w-0">
              {planName || `Plan ${startDate}`} · {plan.days.length} día
              {plan.days.length === 1 ? "" : "s"}
            </span>
            <button
              className="btn-ghost"
              onClick={exportPdf}
              title="Guarda el plan en un archivo PDF"
            >
              📄 Exportar PDF
            </button>
            <button
              className="btn-primary"
              disabled={savePlan.isPending}
              onClick={() => savePlan.mutate()}
              title="Guarda el plan en la lista de abajo"
            >
              {savePlan.isPending
                ? t.common.cargando
                : currentPlanId
                  ? "💾 Actualizar"
                  : "💾 Guardar"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
