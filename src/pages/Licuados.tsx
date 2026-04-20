import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";

const MEALS = [
  { key: "desayuno", label: t.meals.desayuno },
  { key: "colacion1", label: t.meals.colacion1 },
  { key: "comida", label: t.meals.comida },
  { key: "colacion2", label: t.meals.colacion2 },
  { key: "cena", label: t.meals.cena },
];

export default function Licuados() {
  const qc = useQueryClient();
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const { data: hasKey } = useQuery({
    queryKey: ["has_key"],
    queryFn: api.settingsHasKey,
  });
  const [text, setText] = useState("");
  const [mealType, setMealType] = useState("desayuno");
  const [error, setError] = useState<string | null>(null);

  const { data: smoothies } = useQuery({
    queryKey: ["smoothies", activeUserId],
    queryFn: () => api.smoothiesList(activeUserId!),
    enabled: activeUserId != null,
  });

  const parseMut = useMutation({
    mutationFn: () => api.smoothieParseAndSave(activeUserId!, mealType, text),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["smoothies"] });
      setText("");
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const deleteMut = useMutation({
    mutationFn: (id: number) => api.smoothieDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["smoothies"] }),
  });

  function onAnalyze() {
    setError(null);
    if (!activeUserId) {
      setError("Selecciona un usuario activo arriba antes de guardar su licuado.");
      return;
    }
    if (!text.trim()) {
      setError("Escribe los ingredientes del licuado antes de analizar.");
      return;
    }
    if (!hasKey) {
      setError("Falta configurar la clave de OpenAI. Ve a Ajustes → Clave IA.");
      return;
    }
    parseMut.mutate();
  }

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">
          {t.common.seleccionarUsuario} para registrar y analizar sus licuados.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <p className="text-sm text-mint-700">{t.smoothie.subtitulo}</p>

      <div className="card space-y-3">
        <div className="flex gap-3">
          <label className="space-y-1 w-48">
            <span className="label">{t.smoothie.momento}</span>
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
        </div>
        <textarea
          className="input min-h-[120px]"
          placeholder={t.smoothie.placeholder}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        {!hasKey && (
          <p className="text-xs text-amber-700 bg-amber-50 p-2 rounded">
            ⚠ Configura la clave de OpenAI en Ajustes → Clave IA antes de analizar licuados.
          </p>
        )}
        {error && <p className="text-sm text-red-600">{error}</p>}
        <div className="flex justify-end">
          <button
            className="btn-primary"
            disabled={parseMut.isPending}
            onClick={onAnalyze}
          >
            {parseMut.isPending ? t.common.cargando : t.smoothie.analizar}
          </button>
        </div>
      </div>

      <div className="space-y-3">
        {smoothies && smoothies.length > 0 ? (
          smoothies.map((s) => (
            <div key={s.id} className="card space-y-2">
              <div className="flex items-center justify-between">
                <div>
                  <span className="chip">{s.meal_type}</span>
                  <div className="text-sm mt-2 italic text-mint-700">"{s.raw_text}"</div>
                </div>
                <button className="btn-danger" onClick={() => deleteMut.mutate(s.id)}>
                  {t.common.eliminar}
                </button>
              </div>
              {s.parsed && (
                <table className="table">
                  <thead>
                    <tr>
                      <th>{t.common.nombre}</th>
                      <th>{t.common.cantidad}</th>
                      <th>{t.common.unidad}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {s.parsed.ingredients.map((ing, i) => (
                      <tr key={i}>
                        <td>{ing.name}</td>
                        <td>{ing.quantity}</td>
                        <td>{ing.unit}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          ))
        ) : (
          <p className="text-sm text-mint-700">{t.smoothie.vacio}</p>
        )}
      </div>
    </div>
  );
}
