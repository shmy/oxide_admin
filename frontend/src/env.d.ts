/// <reference types="@rsbuild/core/types" />

/**
 * Imports the SVG file as a React component.
 * @requires [@rsbuild/plugin-svgr](https://npmjs.com/package/@rsbuild/plugin-svgr)
 */
declare module '*.svg?react' {
  import type React from 'react';
  const ReactComponent: React.FunctionComponent<React.SVGProps<SVGSVGElement>>;
  export default ReactComponent;
}

declare var amisScoped: {
  doAction: (action: any) => void;
};
declare var amisRequire: (name: string) => any;
declare var _permissions: Set<number>;
declare var _hasPermission: (permission: number) => boolean;
declare var _j: (schema: Record<string, unknown>) => void;
declare var _t: (key: string, args?: Record<string, unknown>) => string;