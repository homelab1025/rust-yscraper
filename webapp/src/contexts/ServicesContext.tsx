import { createContext, useContext, type ReactNode } from 'react';
import { CrateApiLinksApi, CrateApiCommentsApi } from '../api-client';
import { apiConfig } from '../api-config';

interface Services {
  linksApi: CrateApiLinksApi;
  commentsApi: CrateApiCommentsApi;
}

const ServicesContext = createContext<Services | null>(null);

// Module-scope singletons — created once, never recreated on re-renders
const services: Services = {
  linksApi: new CrateApiLinksApi(apiConfig),
  commentsApi: new CrateApiCommentsApi(apiConfig),
};

export function ServicesProvider({ children }: { children: ReactNode }) {
  return (
    <ServicesContext.Provider value={services}>
      {children}
    </ServicesContext.Provider>
  );
}

export function useServices(): Services {
  const ctx = useContext(ServicesContext);
  if (ctx === null) {
    throw new Error('useServices must be used within a ServicesProvider');
  }
  return ctx;
}
