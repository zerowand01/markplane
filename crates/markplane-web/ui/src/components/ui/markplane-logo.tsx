import { type SVGProps } from "react";

export function MarkplaneLogo(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="-85 -100 170 200"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      <line x1="-70" y1="-25" x2="70" y2="-25" stroke="currentColor" strokeWidth="16" strokeLinecap="round" />
      <line x1="-70" y1="25" x2="70" y2="25" stroke="currentColor" strokeWidth="16" strokeLinecap="round" />
      <path d="M-60,85 Q10,0 -60,-85" stroke="currentColor" strokeWidth="16" strokeLinecap="round" />
      <path d="M-10,85 Q60,0 -10,-85" stroke="currentColor" strokeWidth="16" strokeLinecap="round" />
    </svg>
  );
}
