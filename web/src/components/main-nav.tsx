"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { siteConfig } from "@/config/site";
import { cn } from "@/lib/utils";
import { Icons } from "@/components/icons";
import { f1 } from "@/app/fonts";

export function MainNav() {
  const pathname = usePathname();

  return (
    <div className="mr-4 hidden md:flex">
      <Link href="/" className="mr-4 flex items-center space-x-2 lg:mr-6">
        <span className="hidden font-bold lg:inline-block">
          <span className={f1.className}>{siteConfig.name}</span>
        </span>
      </Link>
      <nav className="flex items-center gap-4 text-sm lg:gap-6">
        <Link
          href="/projects"
          className={cn(
            "hover:text-foreground/80 transition-colors",
            pathname === "/projects" ? "text-foreground" : "text-foreground/60",
          )}
        >
          Projects
        </Link>
        <Link
          href="/org"
          className={cn(
            "hover:text-foreground/80 transition-colors",
            pathname?.startsWith("/org")
              ? "text-foreground"
              : "text-foreground/60",
          )}
        >
          Organisation
        </Link>
        <Link
          href="/blog"
          className={cn(
            "hover:text-foreground/80 transition-colors",
            pathname?.startsWith("/blog")
              ? "text-foreground"
              : "text-foreground/60",
          )}
        >
          Blog
        </Link>
      </nav>
    </div>
  );
}
