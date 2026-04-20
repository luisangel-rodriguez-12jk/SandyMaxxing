import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { api, Food, FoodGroup } from "../api/invoke";
import { t } from "../i18n/es";

export default function Alimentos() {
  const qc = useQueryClient();
  const { data: groups } = useQuery({ queryKey: ["groups"], queryFn: api.foodGroupsList });
  const { data: foods } = useQuery({
    queryKey: ["foods", null],
    queryFn: () => api.foodsList(null),
  });
  const [editing, setEditing] = useState<Partial<Food> | null>(null);
  const [newGroup, setNewGroup] = useState("");

  const createGroup = useMutation({
    mutationFn: (name: string) => api.foodGroupsCreate(name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["groups"] });
      setNewGroup("");
    },
  });
  const deleteGroup = useMutation({
    mutationFn: (id: number) => api.foodGroupsDelete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["groups"] });
      qc.invalidateQueries({ queryKey: ["foods"] });
    },
  });
  const saveFood = useMutation({
    mutationFn: async (f: Partial<Food>) => {
      if (f.id) {
        await api.foodsUpdate({
          id: f.id,
          group_id: f.group_id!,
          name: f.name!,
          portion_quantity: f.portion_quantity!,
          portion_unit: f.portion_unit!,
        });
      } else {
        await api.foodsCreate({
          group_id: f.group_id!,
          name: f.name!,
          portion_quantity: f.portion_quantity!,
          portion_unit: f.portion_unit!,
        });
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["foods"] });
      setEditing(null);
    },
  });
  const deleteFood = useMutation({
    mutationFn: (id: number) => api.foodsDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["foods"] }),
  });

  const byGroup = useMemo(() => {
    const map = new Map<string, Food[]>();
    (foods ?? []).forEach((f) => {
      const list = map.get(f.group_name) ?? [];
      list.push(f);
      map.set(f.group_name, list);
    });
    return Array.from(map.entries());
  }, [foods]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-end">
        <button
          className="btn-primary"
          onClick={() => setEditing({ portion_quantity: 1, portion_unit: "pieza" })}
        >
          {t.foods.nuevo}
        </button>
      </div>

      <div className="card">
        <h2 className="font-medium mb-3">{t.foods.gruposTitulo}</h2>
        <div className="flex flex-wrap gap-2 mb-3">
          {groups?.map((g) => (
            <span key={g.id} className="chip">
              {g.name}
              <button
                onClick={() => deleteGroup.mutate(g.id)}
                className="text-mint-700/60 hover:text-red-600"
                title={t.common.eliminar}
              >
                ×
              </button>
            </span>
          ))}
        </div>
        <div className="flex gap-2 max-w-md">
          <input
            className="input"
            placeholder={t.foods.nuevoGrupo}
            value={newGroup}
            onChange={(e) => setNewGroup(e.target.value)}
          />
          <button
            className="btn-primary"
            disabled={!newGroup.trim()}
            onClick={() => createGroup.mutate(newGroup.trim())}
          >
            {t.common.agregar}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
        {byGroup.map(([groupName, list]) => (
          <div key={groupName} className="card">
            <h3 className="font-semibold text-mint-700 mb-2">{groupName}</h3>
            <table className="table">
              <thead>
                <tr>
                  <th>{t.common.nombre}</th>
                  <th>{t.common.porcion}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {list.map((f) => (
                  <tr key={f.id}>
                    <td>{f.name}</td>
                    <td>
                      {f.portion_quantity} {f.portion_unit}
                    </td>
                    <td className="text-right space-x-2">
                      <button className="btn-ghost" onClick={() => setEditing(f)}>
                        {t.common.editar}
                      </button>
                      <button className="btn-danger" onClick={() => deleteFood.mutate(f.id)}>
                        ×
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ))}
      </div>

      {editing && groups && (
        <FoodForm
          value={editing}
          groups={groups}
          onCancel={() => setEditing(null)}
          onSave={(f) => saveFood.mutate(f)}
        />
      )}
    </div>
  );
}

function FoodForm({
  value,
  groups,
  onSave,
  onCancel,
}: {
  value: Partial<Food>;
  groups: FoodGroup[];
  onSave: (f: Partial<Food>) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState<Partial<Food>>({
    ...value,
    group_id: value.group_id ?? groups[0]?.id,
  });
  const [error, setError] = useState<string | null>(null);

  function submit() {
    if (!(form.name ?? "").trim()) {
      setError("El nombre del alimento es obligatorio.");
      return;
    }
    if (!form.group_id) {
      setError("Selecciona un grupo para el alimento.");
      return;
    }
    if (!form.portion_quantity || form.portion_quantity <= 0) {
      setError("Ingresa una cantidad de porción mayor a 0.");
      return;
    }
    if (!(form.portion_unit ?? "").trim()) {
      setError("Ingresa la unidad de la porción (p. ej. pza, taza, gramos).");
      return;
    }
    setError(null);
    onSave(form);
  }

  return (
    <div className="card space-y-3 max-w-xl">
      <h2 className="font-medium">{value.id ? t.common.editar : t.foods.nuevo}</h2>
      <div className="grid grid-cols-2 gap-3">
        <label className="space-y-1">
          <span className="label">{t.common.nombre}</span>
          <input
            className="input"
            value={form.name ?? ""}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
        </label>
        <label className="space-y-1">
          <span className="label">{t.common.grupo}</span>
          <select
            className="input"
            value={form.group_id ?? ""}
            onChange={(e) => setForm({ ...form, group_id: Number(e.target.value) })}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.name}
              </option>
            ))}
          </select>
        </label>
        <label className="space-y-1">
          <span className="label">{t.common.cantidad}</span>
          <input
            type="number"
            step="0.01"
            className="input"
            value={form.portion_quantity ?? ""}
            onChange={(e) => setForm({ ...form, portion_quantity: Number(e.target.value) })}
          />
        </label>
        <label className="space-y-1">
          <span className="label">{t.common.unidad}</span>
          <input
            className="input"
            value={form.portion_unit ?? ""}
            onChange={(e) => setForm({ ...form, portion_unit: e.target.value })}
          />
        </label>
      </div>
      {error && <p className="text-sm text-red-600">{error}</p>}
      <div className="flex justify-end gap-2">
        <button className="btn-ghost" onClick={onCancel}>
          {t.common.cancelar}
        </button>
        <button className="btn-primary" onClick={submit}>
          {t.common.guardar}
        </button>
      </div>
    </div>
  );
}
