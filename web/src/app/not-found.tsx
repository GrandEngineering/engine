import Link from "next/link";
import { Separator } from "@/components/ui/separator";

export default function NotFound() {
  return (
    <div className="mt-10 flex-1 place-self-center self-center justify-self-center text-xl">
      404 | The page you were looking for was not found :(
      <Separator />
    </div>
  );
}
