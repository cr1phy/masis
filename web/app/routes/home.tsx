import type { Route } from "./+types/home";
import Registration from "./registration";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "This is a test" },
    { name: "description", content: "Welcome to React Router!" },
  ];
}

export default function Home() {
  return <Registration />;
}
