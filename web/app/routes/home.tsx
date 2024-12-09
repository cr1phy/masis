import type { Route } from "./+types/home";
import { Welcome } from "../welcome/welcome";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Lox" },
    { name: "description", content: "Welcome to React Router!" },
    { name:  }
  ];
}

export default function Home() {
  return <Welcome />;
}
