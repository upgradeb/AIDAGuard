import { cn } from "@/lib/utils";

interface LogoProps {
  size?: number;
  collapsed?: boolean;
}

export default function Logo({ size = 32, collapsed = false }: LogoProps) {
  return (
    <div className={cn("flex items-center select-none", collapsed ? "" : "gap-2.5")}>
      <img
        src="/logo.png"
        alt="AIDAGuard"
        className="rounded-[22%]"
        style={{ width: size, height: size }}
      />
      {!collapsed && (
        <h1 className="text-xl font-bold whitespace-nowrap m-0">
          <span className="text-preset">AIDA</span>
          <span className="text-foreground">Guard</span>
        </h1>
      )}
    </div>
  );
}
