import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { api } from "../api/invoke";
import { t } from "../i18n/es";

export default function Familia() {
  const qc = useQueryClient();
  const { data: users } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const { data: plans } = useQuery({ queryKey: ["family"], queryFn: api.familyList });
  const [name, setName] = useState("");
  const [weekStart, setWeekStart] = useState(new Date().toISOString().slice(0, 10));
  const [selected, setSelected] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);

  const createMut = useMutation({
    mutationFn: () => api.familyCreate(name, weekStart, selected),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["family"] });
      setName("");
      setSelected([]);
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });
  const deleteMut = useMutation({
    mutationFn: (id: number) => api.familyDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["family"] }),
  });

  function onCreate() {
    setError(null);
    if (!name.trim()) {
      setError("Dale un nombre al plan familiar antes de guardar.");
      return;
    }
    if (selected.length === 0) {
      setError("Selecciona al menos un miembro de la familia.");
      return;
    }
    createMut.mutate();
  }

  const { data: compat } = useQuery({
    queryKey: ["compat", selected],
    queryFn: () => api.familyCompatibility(selected),
    enabled: selected.length > 0,
  });

  const byGroup = useMemo(() => {
    const map = new Map<string, typeof compat extends null ? never : any>();
    (compat?.allowed ?? []).forEach((f: any) => {
      const arr = map.get(f.group_name) ?? [];
      arr.push(f);
      map.set(f.group_name, arr);
    });
    return Array.from(map.entries());
  }, [compat]);

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-2xl font-semibold">{t.familia.titulo}</h1>
      </header>

      <div className="card space-y-3">
        <h2 className="font-medium">{t.familia.nuevo}</h2>
        <div className="grid grid-cols-2 gap-3">
          <label className="space-y-1">
            <span className="label">{t.common.nombre}</span>
            <input
              className="input"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <label className="space-y-1">
            <span className="label">{t.dietPlanner.semana}</span>
            <input
              type="date"
              className="input"
              value={weekStart}
              onChange={(e) => setWeekStart(e.target.value)}
            />
          </label>
        </div>
        <div>
          <span className="label">{t.familia.miembros}</span>
          <div className="flex flex-wrap gap-2 mt-1">
            {users?.map((u) => {
              const on = selected.includes(u.id);
              return (
                <button
                  key={u.id}
                  onClick={() =>
                    setSelected((s) =>
                      on ? s.filter((i) => i !== u.id) : [...s, u.id]
                    )
                  }
                  className={
                    on
                      ? "rounded-full px-3 py-1 text-xs font-medium bg-mint-600 text-white"
                      : "rounded-full px-3 py-1 text-xs font-medium bg-mint-100 text-mint-800"
                  }
                >
                  {u.name}
                </button>
              );
            })}
          </div>
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        <div className="flex justify-end">
          <button
            className="btn-primary"
            disabled={createMut.isPending}
            onClick={onCreate}
          >
            {createMut.isPending ? t.common.cargando : t.common.guardar}
          </button>
        </div>
      </div>

      {compat && selected.length > 0 && (
        <div className="card">
          <h2 className="font-medium mb-3">{t.familia.compatibles}</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
            {byGroup.map(([gname, foods]: any) => (
              <div key={gname}>
                <h4 className="text-sm font-semibold text-mint-700">{gname}</h4>
                <div className="flex flex-wrap gap-1 mt-1">
                  {foods.map((f: any) => (
                    <span key={f.id} className="chip">
                      {f.name}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="space-y-3">
        {plans?.map((p) => (
          <div key={p.id} className="card flex justify-between items-center">
            <div>
              <div className="font-medium">{p.name}</div>
              <div className="text-xs text-mint-600">
                {p.week_start} · {p.user_ids.length} miembros
              </div>
            </div>
            <button className="btn-danger" onClick={() => deleteMut.mutate(p.id)}>
              {t.common.eliminar}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
