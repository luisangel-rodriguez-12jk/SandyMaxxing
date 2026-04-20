import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { api, Measurement } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";

function todayISO(): string {
  return new Date().toISOString().slice(0, 10);
}

export default function Mediciones() {
  const qc = useQueryClient();
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const { data: list } = useQuery({
    queryKey: ["measurements", activeUserId],
    queryFn: () => api.measurementsList(activeUserId!),
    enabled: activeUserId != null,
  });

  const [form, setForm] = useState<Partial<Measurement>>({
    date: todayISO(),
  });
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const hasAnyValue = (f: Partial<Measurement>) =>
    f.weight != null ||
    f.back_cm != null ||
    f.waist_cm != null ||
    f.abdomen_cm != null ||
    f.hip_cm != null;

  const addMut = useMutation({
    mutationFn: () =>
      api.measurementsAdd({
        user_id: activeUserId!,
        date: form.date!,
        weight: form.weight ?? null,
        back_cm: form.back_cm ?? null,
        waist_cm: form.waist_cm ?? null,
        abdomen_cm: form.abdomen_cm ?? null,
        hip_cm: form.hip_cm ?? null,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["measurements"] });
      setForm({ date: todayISO() });
      setError(null);
      setMessage("✓ Medición guardada correctamente");
      setTimeout(() => setMessage(null), 2500);
    },
    onError: (e: any) => setError(String(e)),
  });

  const delMut = useMutation({
    mutationFn: (id: number) => api.measurementsDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["measurements"] }),
  });

  async function exportPdf() {
    if (!activeUserId) return;
    try {
      const bytes = await api.pdfMeasurements(activeUserId);
      const path = await save({
        filters: [{ name: "PDF", extensions: ["pdf"] }],
        defaultPath: "mediciones.pdf",
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

  function onSave() {
    setError(null);
    if (!activeUserId) {
      setError("Selecciona un usuario activo antes de guardar la medición.");
      return;
    }
    if (!form.date) {
      setError("La fecha es obligatoria.");
      return;
    }
    if (!hasAnyValue(form)) {
      setError(
        "Captura al menos un valor (peso o alguna medida corporal) antes de guardar.",
      );
      return;
    }
    addMut.mutate();
  }

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">{t.measurements.sinUsuario}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-4xl">
      {list && list.length > 0 && (
        <div className="flex justify-end">
          <button className="btn-ghost" onClick={exportPdf}>
            {t.common.exportar}
          </button>
        </div>
      )}

      <div className="card space-y-3">
        <h2 className="font-medium">{t.measurements.agregarTitulo}</h2>
        <div className="grid grid-cols-3 gap-3">
          <Field
            label={t.common.fecha}
            type="date"
            value={form.date ?? ""}
            onChange={(v) => setForm({ ...form, date: v })}
          />
          <Field
            label={t.measurements.peso}
            value={form.weight?.toString() ?? ""}
            onChange={(v) => setForm({ ...form, weight: v ? Number(v) : null })}
          />
          <Field
            label={t.measurements.espalda}
            value={form.back_cm?.toString() ?? ""}
            onChange={(v) => setForm({ ...form, back_cm: v ? Number(v) : null })}
          />
          <Field
            label={t.measurements.cintura}
            value={form.waist_cm?.toString() ?? ""}
            onChange={(v) => setForm({ ...form, waist_cm: v ? Number(v) : null })}
          />
          <Field
            label={t.measurements.abdomen}
            value={form.abdomen_cm?.toString() ?? ""}
            onChange={(v) => setForm({ ...form, abdomen_cm: v ? Number(v) : null })}
          />
          <Field
            label={t.measurements.cadera}
            value={form.hip_cm?.toString() ?? ""}
            onChange={(v) => setForm({ ...form, hip_cm: v ? Number(v) : null })}
          />
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        {message && <p className="text-sm text-green-700">{message}</p>}
        <div className="flex justify-end">
          <button
            className="btn-primary"
            disabled={addMut.isPending}
            onClick={onSave}
          >
            {addMut.isPending ? t.common.cargando : t.common.guardar}
          </button>
        </div>
      </div>

      <div className="card overflow-x-auto">
        <table className="table min-w-[720px]">
          <thead>
            <tr>
              <th>{t.common.fecha}</th>
              <th>{t.measurements.peso}</th>
              <th>{t.measurements.espalda}</th>
              <th>{t.measurements.cintura}</th>
              <th>{t.measurements.abdomen}</th>
              <th>{t.measurements.cadera}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {list?.map((m) => (
              <tr key={m.id}>
                <td>{m.date}</td>
                <td>{m.weight ?? "—"}</td>
                <td>{m.back_cm ?? "—"}</td>
                <td>{m.waist_cm ?? "—"}</td>
                <td>{m.abdomen_cm ?? "—"}</td>
                <td>{m.hip_cm ?? "—"}</td>
                <td className="text-right">
                  <button
                    className="btn-danger"
                    onClick={() => {
                      if (confirm(`¿Eliminar la medición del ${m.date}?`)) {
                        delMut.mutate(m.id);
                      }
                    }}
                  >
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
  type = "number",
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: string;
}) {
  return (
    <label className="space-y-1">
      <span className="label">{label}</span>
      <input
        type={type}
        step="0.1"
        className="input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  );
}
