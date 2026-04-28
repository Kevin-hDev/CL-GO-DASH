import scrollDownIcon from "@/assets/fleche.png";

export function ScrollBottomButton({ onClick }: { onClick: () => void }) {
  return (
    <button className="scroll-bottom-btn" onClick={onClick}>
      <img src={scrollDownIcon} alt="" style={{ width: 20, height: 20 }} />
    </button>
  );
}
