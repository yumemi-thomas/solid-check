interface Props {
  title: string;
}


export function Card({title}: Props) {
  return <h1>{title}</h1>;
}
