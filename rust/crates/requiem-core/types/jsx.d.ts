declare const elementBrand: unique symbol;

declare namespace JSX {
  interface Element {
    readonly [elementBrand]: true;
  }
  type Children = Element | boolean | null | undefined | Children[];
  interface IntrinsicAttributes {
    key?: string | number;
  }
  interface ElementChildrenAttribute {
    children: {};
  }
  interface IntrinsicElements {
    div: {
      key?: string | number;
      className?: string;
      style?: string;
      onClick?: () => void;
      onContextMenu?: (event: { x: number; y: number }) => void;
      children?: Children;
    };
  }
}
