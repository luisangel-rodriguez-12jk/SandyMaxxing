import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api, User } from "../api/invoke";
import { t } from "../i18n/es";

export default function Usuarios() {
  const qc = useQueryClient();
  const { data: users, isLoading } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const [editing, setEditing] = useState<Partial<User> | null>(null);

  const createMut = useMutation({
    mutationFn: (u: Partial<User> & { name: string }) => api.usersCreate(u),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
  const updateMut = useMutation({
    mutationFn: (u: User) => api.usersUpdate(u),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
  const deleteMut = useMutation({
    mutationFn: (id: number) => api.usersDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });

  function save(form: Partial<User>) {
    const data = {
      name: form.name ?? "",
      age: form.age ?? null,
      height: form.height ?? null,
      sex: form.sex ?? null,
    };
    if (!data.name.trim()) return;
    if (form.id) {
      updateMut.mutate({ id: form.id, ...data });
    } else {
      createMut.mutate(data);
    }
    setEditing(null);
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <header className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">{t.nav.usuarios}</h1>
        <button className="btn-primary" onClick={() => setEditing({})}>
          {t.common.agregar}
        </button>
      </header>

      <div className="card">
        {isLoading ? (
          <p>{t.common.cargando}</p>
        ) : users && users.length > 0 ? (
          <table className="table">
            <thead>
              <tr>
                <th>{t.common.nombre}</th>
                <th>{t.common.edad}</th>
                <th>{t.common.altura}</th>
                <th>{t.common.sexo}</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {users.map((u) => (
                <tr key={u.id}>
                  <td>{u.name}</td>
                  <td>{u.age ?? "—"}</td>
                  <td>{u.height ?? "—"}</td>
                  <td>{u.sex ?? "—"}</td>
                  <td className="text-right space-x-2">
                    <button className="btn-ghost" onClick={() => setEditing(u)}>
                      {t.common.editar}
                    </button>
                    <button className="btn-danger" onClick={() => deleteMut.mutate(u.id)}>
                      {t.common.eliminar}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="text-sm text-mint-700">{t.common.vacio}</p>
        )}
      </div>

      {editing && (
        <UserForm
          value={editing}
          onCancel={() => setEditing(null)}
          onSave={save}
        />
      )}
    </div>
  );
}

function UserForm({
  value,
  onSave,
  onCancel,
}: {
  value: Partial<User>;
  onSave: (u: Partial<User>) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState<Partial<User>>(value);
  const [error, setError] = useState<string | null>(null);

  function submit() {
    if (!(form.name ?? "").trim()) {
      setError("El nombre es obligatorio.");
      return;
    }
    setError(null);
    onSave(form);
  }

  return (
    <div className="card space-y-4 max-w-lg">
      <h2 className="font-medium">{value.id ? t.common.editar : t.common.agregar}</h2>
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
          <span className="label">{t.common.edad}</span>
          <input
            type="number"
            className="input"
            value={form.age ?? ""}
            onChange={(e) =>
              setForm({ ...form, age: e.target.value ? Number(e.target.value) : null })
            }
          />
        </label>
        <label className="space-y-1">
          <span className="label">{t.common.altura}</span>
          <input
            type="number"
            className="input"
            value={form.height ?? ""}
            onChange={(e) =>
              setForm({
                ...form,
                height: e.target.value ? Number(e.target.value) : null,
              })
            }
          />
        </label>
        <label className="space-y-1">
          <span className="label">{t.common.sexo}</span>
          <select
            className="input"
            value={form.sex ?? ""}
            onChange={(e) => setForm({ ...form, sex: e.target.value || null })}
          >
            <option value="">—</option>
            <option value="F">Femenino</option>
            <option value="M">Masculino</option>
            <option value="O">Otro</option>
          </select>
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
