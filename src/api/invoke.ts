import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export type User = {
  id: number;
  name: string;
  age: number | null;
  height: number | null;
  sex: string | null;
};

export type Measurement = {
  id: number;
  user_id: number;
  date: string;
  weight: number | null;
  back_cm: number | null;
  waist_cm: number | null;
  abdomen_cm: number | null;
  hip_cm: number | null;
};

export type FoodGroup = { id: number; name: string };

export type Food = {
  id: number;
  group_id: number;
  group_name: string;
  name: string;
  portion_quantity: number;
  portion_unit: string;
  forbidden: boolean;
};

export type DietPortion = {
  meal_type: string;
  group_id: number;
  group_name: string;
  portions: number;
};

export type WeeklyDiet = {
  id: number;
  user_id: number;
  week_start: string;
  portions: DietPortion[];
};

export type ParsedSmoothie = {
  ingredients: { name: string; quantity: number; unit: string }[];
};

export type Smoothie = {
  id: number;
  user_id: number;
  meal_type: string;
  raw_text: string;
  parsed: ParsedSmoothie | null;
};

export type PlanIngredient = { name: string; quantity: number; unit: string };
export type GroupPortion = { group: string; portions: number };
export type PlanUserPortion = {
  user: string;
  notes: string;
  /** Conteo explícito de porciones por grupo que consume ese usuario en esa comida.
   * Puede no venir si el plan fue generado con una versión vieja. */
  portions_consumed?: GroupPortion[];
};
export type PlanMeal = {
  meal_type: string;
  name: string;
  instructions: string;
  ingredients: PlanIngredient[];
  per_user_portions: PlanUserPortion[];
};
export type PlanDay = { day: string; meals: PlanMeal[] };
export type PlanResult = { days: PlanDay[] };
export type SingleMeal = {
  name: string;
  instructions: string;
  ingredients: PlanIngredient[];
  per_user_portions: PlanUserPortion[];
};

export type MealOptions = { options: SingleMeal[] };

export type SavedPlan = {
  id: number;
  name: string;
  week_start: string;
  user_ids_json: string;
  plan_json: string;
  notes: string | null;
  created_at: string;
};

export type ShoppingItem = {
  name: string;
  group_name: string;
  quantity: number;
  unit: string;
};

export type FamilyPlan = {
  id: number;
  name: string;
  week_start: string;
  user_ids: number[];
};

export type CompatPayload = {
  allowed: Food[];
  forbidden_by_user: { user_id: number; foods: string[] }[];
};

export const api = {
  usersList: () => tauriInvoke<User[]>("users_list"),
  usersCreate: (p: Partial<User> & { name: string }) =>
    tauriInvoke<User>("users_create", p),
  usersUpdate: (p: User) => tauriInvoke<User>("users_update", p),
  usersDelete: (id: number) => tauriInvoke<void>("users_delete", { id }),

  measurementsList: (user_id: number) =>
    tauriInvoke<Measurement[]>("measurements_list", { userId: user_id }),
  measurementsAdd: (p: Omit<Measurement, "id">) =>
    tauriInvoke<number>("measurements_add", {
      userId: p.user_id,
      date: p.date,
      weight: p.weight,
      backCm: p.back_cm,
      waistCm: p.waist_cm,
      abdomenCm: p.abdomen_cm,
      hipCm: p.hip_cm,
    }),
  measurementsDelete: (id: number) =>
    tauriInvoke<void>("measurements_delete", { id }),

  foodGroupsList: () => tauriInvoke<FoodGroup[]>("food_groups_list"),
  foodGroupsCreate: (name: string) =>
    tauriInvoke<FoodGroup>("food_groups_create", { name }),
  foodGroupsDelete: (id: number) =>
    tauriInvoke<void>("food_groups_delete", { id }),

  foodsList: (user_id: number | null) =>
    tauriInvoke<Food[]>("foods_list", { userId: user_id }),
  foodsCreate: (p: {
    group_id: number;
    name: string;
    portion_quantity: number;
    portion_unit: string;
  }) =>
    tauriInvoke<number>("foods_create", {
      groupId: p.group_id,
      name: p.name,
      portionQuantity: p.portion_quantity,
      portionUnit: p.portion_unit,
    }),
  foodsUpdate: (p: {
    id: number;
    group_id: number;
    name: string;
    portion_quantity: number;
    portion_unit: string;
  }) =>
    tauriInvoke<void>("foods_update", {
      id: p.id,
      groupId: p.group_id,
      name: p.name,
      portionQuantity: p.portion_quantity,
      portionUnit: p.portion_unit,
    }),
  foodsDelete: (id: number) => tauriInvoke<void>("foods_delete", { id }),
  forbiddenSet: (user_id: number, food_id: number, forbidden: boolean) =>
    tauriInvoke<void>("forbidden_set", {
      userId: user_id,
      foodId: food_id,
      forbidden,
    }),

  dietGet: (user_id: number, week_start: string) =>
    tauriInvoke<WeeklyDiet>("diet_get", { userId: user_id, weekStart: week_start }),
  dietSetPortion: (p: {
    diet_id: number;
    meal_type: string;
    group_id: number;
    portions: number;
  }) =>
    tauriInvoke<void>("diet_set_portion", {
      dietId: p.diet_id,
      mealType: p.meal_type,
      groupId: p.group_id,
      portions: p.portions,
    }),

  smoothiesList: (user_id: number) =>
    tauriInvoke<Smoothie[]>("smoothies_list", { userId: user_id }),
  smoothieParseAndSave: (user_id: number, meal_type: string, raw_text: string) =>
    tauriInvoke<Smoothie>("smoothie_parse_and_save", {
      userId: user_id,
      mealType: meal_type,
      rawText: raw_text,
    }),
  smoothieDelete: (id: number) => tauriInvoke<void>("smoothie_delete", { id }),

  planGenerate: (
    user_ids: number[],
    week_start: string,
    end_date: string | null,
    notes: string | null,
  ) =>
    tauriInvoke<PlanResult>("plan_generate", {
      userIds: user_ids,
      weekStart: week_start,
      endDate: end_date,
      notes,
    }),
  mealDesign: (
    user_ids: number[],
    week_start: string,
    notes: string | null,
    meal_type: string | null,
  ) =>
    tauriInvoke<SingleMeal>("meal_design", {
      userIds: user_ids,
      weekStart: week_start,
      notes,
      mealType: meal_type,
    }),
  mealOptions: (
    user_ids: number[],
    week_start: string,
    notes: string | null,
    meal_type: string,
    count: number,
    exclude_names: string[],
  ) =>
    tauriInvoke<MealOptions>("meal_options", {
      userIds: user_ids,
      weekStart: week_start,
      notes,
      mealType: meal_type,
      count,
      excludeNames: exclude_names,
    }),
  planTweakMeal: (
    user_ids: number[],
    week_start: string,
    day: string,
    original: PlanMeal,
    user_instruction: string,
  ) =>
    tauriInvoke<PlanMeal>("plan_tweak_meal", {
      userIds: user_ids,
      weekStart: week_start,
      day,
      original,
      userInstruction: user_instruction,
    }),

  savedPlansList: () => tauriInvoke<SavedPlan[]>("saved_plans_list"),
  savedPlansGet: (id: number) => tauriInvoke<SavedPlan>("saved_plans_get", { id }),
  savedPlansUpsert: (
    id: number | null,
    name: string,
    week_start: string,
    user_ids: number[],
    plan: PlanResult,
    notes: string | null,
  ) =>
    tauriInvoke<number>("saved_plans_upsert", {
      id,
      name,
      weekStart: week_start,
      userIds: user_ids,
      plan,
      notes,
    }),
  savedPlansDelete: (id: number) => tauriInvoke<void>("saved_plans_delete", { id }),

  familyList: () => tauriInvoke<FamilyPlan[]>("family_plans_list"),
  familyCreate: (name: string, week_start: string, user_ids: number[]) =>
    tauriInvoke<number>("family_plans_create", {
      name,
      weekStart: week_start,
      userIds: user_ids,
    }),
  familyDelete: (id: number) => tauriInvoke<void>("family_plans_delete", { id }),
  familyCompatibility: (user_ids: number[]) =>
    tauriInvoke<CompatPayload>("family_compatibility", { userIds: user_ids }),

  shoppingBuild: (user_ids: number[], plan: PlanResult | null) =>
    tauriInvoke<ShoppingItem[]>("shopping_build", {
      userIds: user_ids,
      plan,
    }),

  pdfPlan: (plan: PlanResult, title: string) =>
    tauriInvoke<number[]>("pdf_plan", { plan, title }),
  pdfShopping: (items: ShoppingItem[], title: string) =>
    tauriInvoke<number[]>("pdf_shopping", { items, title }),
  pdfMeasurements: (user_id: number) =>
    tauriInvoke<number[]>("pdf_measurements", { userId: user_id }),

  settingsSetKey: (key: string) =>
    tauriInvoke<void>("settings_set_openai_key", { key }),
  settingsHasKey: () => tauriInvoke<boolean>("settings_has_openai_key"),
  settingsClearKey: () => tauriInvoke<void>("settings_clear_openai_key"),
};
