import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, Recipe, SingleMeal } from "../api/invoke";
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

export default function DisenarComida() {
  const qc = useQueryClient();
  const { data: users } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const { data: hasKey } = useQuery({
    queryKey: ["has_key"],
    queryFn: api.settingsHasKey,
  });
  const [selected, setSelected] = useState<number[]>([]);
  const [mealType, setMealType] = useState("comida");
  const [notes, setNotes] = useState("");
  const [options, setOptions] = useState<SingleMeal[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [savedIdxs, setSavedIdxs] = useState<Set<number>>(new Set());
  const [expandedFav, setExpandedFav] = useState<number | null>(null);

  const weekStart = todayISO();

  const { data: favorites } = useQuery({
    queryKey: ["recipes", mealType],
    queryFn: () => api.recipesList(mealType),
  });

  const fetchOptions = useMutation({
    mutationFn: (opts: { count: number; exclude: string[] }) =>
      api.mealOptions(selected, weekStart, notes || null, mealType, opts.count, opts.exclude),
    onSuccess: (res, vars) => {
      if (vars.exclude.length === 0) {
        setOptions(res.options);
        setSavedIdxs(new Set());
      } else {
        setOptions((prev) => [...prev, ...res.options]);
      }
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const saveFav = useMutation({
    mutationFn: (v: { idx: number; meal: SingleMeal }) =>
      api.recipesSave({
        name: v.meal.name,
        instructions: v.meal.instructions,
        meal_type: mealType,
        ingredients: v.meal.ingredients.map((i) => ({
          name: i.name,
          quantity: i.quantity,
          unit: i.unit,
        })),
      }),
    onSuccess: (_id, vars) => {
      setSavedIdxs((prev) => new Set(prev).add(vars.idx));
      qc.invalidateQueries({ queryKey: ["recipes", mealType] });
    },
    onError: (e: any) => setError(String(e)),
  });

  const deleteFav = useMutation({
    mutationFn: (id: number) => api.recipesDelete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["recipes", mealType] });
    },
  });

  function onGenerate() {
    setError(null);
    if (selected.length === 0) {
      setError("Selecciona al menos un usuario.");
      return;
    }
    if (!hasKey) {
      setError("Falta configurar la clave de OpenAI. Ve a Ajustes → Clave IA.");
      return;
    }
    setOptions([]);
    setSavedIdxs(new Set());
    fetchOptions.mutate({ count: 3, exclude: [] });
  }

  function onGenerateMore() {
    const exclude = options.map((o) => o.name);
    fetchOptions.mutate({ count: 3, exclude });
  }

  const MEAL_LABEL =
    MEALS.find((m) => m.key === mealType)?.label ?? mealType;

  return (
    <div className="space-y-6 max-w-4xl">
      <p className="text-sm text-mint-700">{t.disenar.subtitulo}</p>

      <div className="card space-y-3">
        <div>
          <span className="label">Usuarios</span>
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
                Aún no hay usuarios. Crea uno en la sección Usuarios.
              </span>
            )}
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <label className="space-y-1">
            <span className="label">Tiempo de comida</span>
            <select
              className="input"
              value={mealType}
              onChange={(e) => setMealType(e.target.value)}
            >
              {MEALS.map((m) => (
                <option key={m.key} value={m.key}>
                  {m.label}
                </option>
              ))}
            </select>
          </label>
          <label className="space-y-1">
            <span className="label">{t.plan.notas}</span>
            <input
              className="input"
              placeholder="Ej. algo con picaña, con salmón, rápido, ligero"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </label>
        </div>

        {!hasKey && (
          <p className="text-xs text-amber-700 bg-amber-50 p-2 rounded">
            ⚠ Configura la clave de OpenAI en Ajustes → Clave IA antes de generar.
          </p>
        )}
        {error && <p className="text-sm text-red-600">{error}</p>}

        <div className="flex justify-end">
          <button
            className="btn-primary"
            disabled={fetchOptions.isPending}
            onClick={onGenerate}
          >
            {fetchOptions.isPending ? t.common.cargando : "Generar 3 opciones"}
          </button>
        </div>
      </div>

      {options.length > 0 && (
        <>
          <div className="space-y-3">
            {options.map((m, i) => {
              const isSaved = savedIdxs.has(i);
              return (
                <div key={i} className="card space-y-2">
                  <div className="flex items-baseline gap-2 flex-wrap">
                    <span className="chip">Opción {i + 1}</span>
                    <h2 className="text-lg font-semibold">{m.name}</h2>
                    <button
                      className={
                        "ml-auto text-xs rounded-full px-3 py-1 font-medium transition " +
                        (isSaved
                          ? "bg-rose-100 text-rose-700 cursor-default"
                          : "bg-mint-100 text-mint-800 hover:bg-rose-50 hover:text-rose-700")
                      }
                      disabled={isSaved || saveFav.isPending}
                      onClick={() => saveFav.mutate({ idx: i, meal: m })}
                      title="Guardar esta receta como favorita"
                    >
                      {isSaved ? "♥ Guardada" : "♡ Guardar favorita"}
                    </button>
                  </div>
                  <div>
                    <h3 className="font-medium text-sm mt-2">Preparación</h3>
                    <p className="text-sm whitespace-pre-wrap">{m.instructions}</p>
                  </div>
                  <div>
                    <h3 className="font-medium text-sm">Ingredientes</h3>
                    <ul className="list-disc ml-5 text-sm">
                      {m.ingredients.map((ing, k) => (
                        <li key={k}>
                          {ing.quantity} {ing.unit} · {ing.name}
                        </li>
                      ))}
                    </ul>
                  </div>
                  {m.per_user_portions.length > 0 && (
                    <div>
                      <h3 className="font-medium text-sm">Porciones por persona</h3>
                      <ul className="text-sm space-y-1.5">
                        {m.per_user_portions.map((p, k) => (
                          <li key={k}>
                            <strong>{p.user}:</strong> {p.notes}
                            {p.portions_consumed && p.portions_consumed.length > 0 && (
                              <div className="flex flex-wrap gap-1 mt-1">
                                {p.portions_consumed.map((gp, j) => (
                                  <span
                                    key={j}
                                    className="inline-flex items-center rounded-full bg-mint-100 text-mint-800 px-2 py-0.5 text-[11px]"
                                  >
                                    {gp.group}: {gp.portions}
                                  </span>
                                ))}
                              </div>
                            )}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div className="flex justify-center">
            <button
              className="btn-ghost"
              disabled={fetchOptions.isPending}
              onClick={onGenerateMore}
            >
              {fetchOptions.isPending
                ? t.common.cargando
                : "Generar 3 opciones más (sin repetir)"}
            </button>
          </div>
        </>
      )}

      {favorites && favorites.length > 0 && (
        <div className="card">
          <h3 className="font-semibold text-mint-700 mb-3">
            ♥ Favoritas de {MEAL_LABEL.toLowerCase()}
          </h3>
          <ul className="space-y-2">
            {favorites.map((fav: Recipe) => {
              const open = expandedFav === fav.id;
              return (
                <li
                  key={fav.id}
                  className="border border-mint-100 rounded-lg p-2"
                >
                  <div className="flex items-center justify-between gap-2">
                    <button
                      className="text-left flex-1 min-w-0"
                      onClick={() => setExpandedFav(open ? null : fav.id)}
                    >
                      <div className="font-medium text-sm truncate">{fav.name}</div>
                      <div className="text-[11px] text-mint-700">
                        {fav.ingredients.length} ingrediente
                        {fav.ingredients.length === 1 ? "" : "s"}
                        {fav.created_at ? ` · ${fav.created_at}` : ""}
                      </div>
                    </button>
                    <button
                      className="btn-danger text-xs"
                      onClick={() => {
                        if (confirm(`¿Eliminar "${fav.name}" de tus favoritas?`)) {
                          deleteFav.mutate(fav.id);
                          if (open) setExpandedFav(null);
                        }
                      }}
                    >
                      ×
                    </button>
                  </div>
                  {open && (
                    <div className="mt-2 space-y-2 bg-mint-50 p-3 rounded">
                      <div>
                        <h4 className="font-medium text-xs">Preparación</h4>
                        <p className="text-xs whitespace-pre-wrap">
                          {fav.instructions}
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium text-xs">Ingredientes</h4>
                        <ul className="list-disc ml-5 text-xs">
                          {fav.ingredients.map((ing, k) => (
                            <li key={k}>
                              {ing.quantity} {ing.unit} · {ing.name}
                            </li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
}
