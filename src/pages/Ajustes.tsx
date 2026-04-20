import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/invoke";
import { t } from "../i18n/es";

export default function Ajustes() {
  const qc = useQueryClient();
  const { data: hasKey } = useQuery({
    queryKey: ["has_key"],
    queryFn: api.settingsHasKey,
  });
  const [key, setKey] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const saveMut = useMutation({
    mutationFn: () => api.settingsSetKey(key),
    onSuccess: () => {
      setMsg(t.settings.guardada);
      setError(null);
      setKey("");
      qc.invalidateQueries({ queryKey: ["has_key"] });
      setTimeout(() => setMsg(null), 2500);
    },
    onError: (e: any) => setError(String(e)),
  });
  const clearMut = useMutation({
    mutationFn: () => api.settingsClearKey(),
    onSuccess: () => {
      setMsg(t.settings.eliminada);
      setError(null);
      qc.invalidateQueries({ queryKey: ["has_key"] });
      setTimeout(() => setMsg(null), 2500);
    },
    onError: (e: any) => setError(String(e)),
  });

  function onSave() {
    setError(null);
    if (!key.trim()) {
      setError("Ingresa la clave antes de guardar (debe comenzar con 'sk-').");
      return;
    }
    if (!key.trim().startsWith("sk-")) {
      setError("La clave de OpenAI debe comenzar con 'sk-'. Verifica que esté completa.");
      return;
    }
    saveMut.mutate();
  }

  function onClear() {
    setError(null);
    if (!hasKey) {
      setError("No hay clave guardada para eliminar.");
      return;
    }
    if (confirm("¿Seguro que quieres eliminar la clave guardada?")) {
      clearMut.mutate();
    }
  }

  return (
    <div className="space-y-6 max-w-xl">
      <div className="card space-y-3">
        <h2 className="font-medium">{t.settings.openai}</h2>
        <p className="text-xs text-mint-700">{t.settings.openaiHelp}</p>
        <div className="flex items-center gap-2">
          <span className={hasKey ? "chip" : "chip-danger"}>
            {hasKey ? "Configurada" : "No configurada"}
          </span>
        </div>
        <input
          type="password"
          className="input"
          placeholder={t.settings.openaiPlaceholder}
          value={key}
          onChange={(e) => setKey(e.target.value)}
        />
        {error && <p className="text-sm text-red-600">{error}</p>}
        {msg && <p className="text-sm text-green-700">{msg}</p>}
        <div className="flex gap-2 justify-end">
          <button className="btn-ghost" onClick={onClear}>
            {t.common.eliminar}
          </button>
          <button
            className="btn-primary"
            onClick={onSave}
            disabled={saveMut.isPending}
          >
            {saveMut.isPending ? t.common.cargando : t.common.guardar}
          </button>
        </div>
      </div>
    </div>
  );
}
