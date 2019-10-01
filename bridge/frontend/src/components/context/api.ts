import { createContext, useContext } from "react";
import { Api } from '~api';

export const ApiContext = createContext<Api>(null as any);

export function useApi(): Api {
  const context: Api | null = useContext(ApiContext);

  if (context === null) {
    throw new Error("Store context is not defined");
  }

  return context;
}
