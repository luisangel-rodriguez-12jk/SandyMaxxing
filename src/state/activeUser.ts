import { create } from "zustand";

interface ActiveUserState {
  activeUserId: number | null;
  setActiveUserId: (id: number | null) => void;
}

export const useActiveUser = create<ActiveUserState>((set) => ({
  activeUserId: null,
  setActiveUserId: (id) => set({ activeUserId: id }),
}));
