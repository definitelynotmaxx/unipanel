/** The ambient background: grid, sodium glow and film grain. */
export function Backdrop() {
  return (
    <div className="atmosphere" aria-hidden="true">
      <div className="atmosphere__glow drift" />
      <div className="atmosphere__grid" />
      <div className="atmosphere__noise" />
    </div>
  );
}
