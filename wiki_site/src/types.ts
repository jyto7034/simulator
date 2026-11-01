export interface ContentTag { label: string; relation: 'effect' | 'require' | 'synergy'; note: string; source: string }
export interface Category { id: string; title: string; description: string; icon: string }
export interface Heading { title: string; anchor: string; level: number }
export interface PageMeta {
  id: string; title: string; path: string; category: string; description: string;
  tags?: ContentTag[];
  classifications?: string[];
  entity?: string; part?: string; index?: boolean; tier: string;
  archived?: boolean; legacyPath?: string; currentPath?: string;
  source: string; sourceName: string; sha256: string;
  relic?: { kind: string; summary: string; status: string; facts: { label: string; value: string }[] };
  preview?: { kind: string; summary: string; status: string; facts: { label: string; value: string }[] };
}
export interface WikiPage extends PageMeta { html: string; toc: Heading[] }
export interface Action { id: string; title: string; input: string; result: string; status: string; note: string; source: string; url: string }
export interface Scene {
  id: string; title: string; subtitle: string; kind: string; group: string; entry: string; exit: string;
  images: { file: string; caption: string; url: string }[]; actions: Action[];
  sources: string[]; sourceLinks: string[]; questions: { text: string; source: string; url: string }[];
}
export interface CatalogEntry { id: string; title: string; group: string; presentation: string; states: string[]; coverage: string[] }
export interface SiteData {
  title: string; pages: PageMeta[]; categories: Category[]; scenes: Scene[];
  groups: { id: string; title: string; summary: string }[]; statuses: Record<string, string>;
  catalog: CatalogEntry[]; catalogStates: number; catalogSource: string; uiUpdated: string;
  config: { name: string; description: string; basePath: string; canonicalOrigin: string; indexable: boolean };
}
export interface SearchRecord { id: string; title: string; category: string; path: string; text: string }
export interface BootData { meta: SiteData; page?: WikiPage; prerendered?: boolean }
