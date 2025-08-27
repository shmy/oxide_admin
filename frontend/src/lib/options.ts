type LabelValue<T> = {
  label: string;
  value: T;
};

export const enabledStatuses: LabelValue<boolean>[] = require("./options/enabledStatuses.json");
