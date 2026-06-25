import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, ChevronRight, HelpCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type {
  AgentInteractiveAnswer,
  AgentInteractiveChoiceRequest,
  AgentInteractiveQuestion,
} from "@/types/agent";
import "./interactive-choice-panel.css";

interface InteractiveChoicePanelProps {
  request?: AgentInteractiveChoiceRequest;
}

const OTHER_VALUE = "other";

export function InteractiveChoicePanel({ request }: InteractiveChoicePanelProps) {
  if (!request) return null;
  return <InteractiveChoicePanelInner key={request.id} request={request} />;
}

function InteractiveChoicePanelInner({ request }: { request: AgentInteractiveChoiceRequest }) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);
  const [activeIndex, setActiveIndex] = useState(0);
  const [answers, setAnswers] = useState<AgentInteractiveAnswer[]>([]);
  const [otherText, setOtherText] = useState("");
  const [otherMode, setOtherMode] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const question = request.questions[step];
  const options = useMemo(() => withOther(question, t("interactiveChoice.other")), [question, t]);

  const submitAnswer = useCallback(async (answer: AgentInteractiveAnswer) => {
    if (submitting) return;
    const nextAnswers = [...answers.filter((item) => item.questionIndex !== step), answer];
    if (step + 1 < request.questions.length) {
      setAnswers(nextAnswers);
      setStep((value) => value + 1);
      setActiveIndex(0);
      setOtherText("");
      setOtherMode(false);
      return;
    }
    setSubmitting(true);
    await invoke("respond_to_interactive_choice", { id: request.id, answers: nextAnswers });
  }, [answers, request, step, submitting]);

  const choose = useCallback((label: string) => {
    if (!question) return;
    if (label === OTHER_VALUE) {
      setOtherMode(true);
      return;
    }
    void submitAnswer({
      questionIndex: step,
      selectedLabels: [label],
    });
  }, [question, step, submitAnswer]);

  const submitOther = useCallback(() => {
    const custom = otherText.trim();
    if (!custom) return;
    void submitAnswer({
      questionIndex: step,
      selectedLabels: [OTHER_VALUE],
      customAnswer: custom,
    });
  }, [otherText, step, submitAnswer]);

  const cancel = useCallback(() => {
    void invoke("respond_to_interactive_choice", {
      id: request.id,
      answers: [],
    }).catch(() => undefined);
  }, [request]);

  useEffect(() => {
    if (!question) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((value) => (value - 1 + options.length) % options.length);
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((value) => (value + 1) % options.length);
      } else if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        if (otherMode) submitOther();
        else choose(options[activeIndex]?.label ?? "");
      } else if (event.key === "Escape") {
        event.preventDefault();
        if (otherMode) setOtherMode(false);
        else cancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [activeIndex, cancel, choose, options, otherMode, question, request, submitOther]);

  if (!question) return null;

  return (
    <div className="icp-panel" role="group" aria-label={t("interactiveChoice.title")}>
      <div className="icp-header">
        <HelpCircle className="icp-icon" aria-hidden="true" />
        <span className="icp-step">
          {t("interactiveChoice.step", { current: step + 1, total: request.questions.length })}
        </span>
        <span className="icp-title">{question.header}</span>
      </div>
      <div className="icp-question">{question.question}</div>
      <div className="icp-options">
        {options.map((option, index) => (
          <ChoiceButton
            key={option.label}
            option={option}
            active={index === activeIndex}
            disabled={submitting}
            otherLabel={t("interactiveChoice.otherLabel")}
            recommendedLabel={t("interactiveChoice.recommended")}
            onHover={() => setActiveIndex(index)}
            onChoose={() => choose(option.label)}
          />
        ))}
      </div>
      {otherMode && (
        <div className="icp-other-row">
          <input
            className="icp-other-input"
            value={otherText}
            onChange={(event) => setOtherText(event.target.value)}
            placeholder={t("interactiveChoice.otherPlaceholder")}
            autoFocus
          />
          <button className="icp-submit" type="button" onClick={submitOther} disabled={!otherText.trim()}>
            <Check className="icp-submit-icon" aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}

function withOther(question: AgentInteractiveQuestion | undefined, label: string) {
  return [
    ...(question?.options ?? []),
    { label: OTHER_VALUE, description: label, recommended: false },
  ];
}

function ChoiceButton({
  option,
  active,
  disabled,
  otherLabel,
  recommendedLabel,
  onHover,
  onChoose,
}: {
  option: ReturnType<typeof withOther>[number];
  active: boolean;
  disabled: boolean;
  otherLabel: string;
  recommendedLabel: string;
  onHover: () => void;
  onChoose: () => void;
}) {
  const label = option.label === OTHER_VALUE ? otherLabel : option.label;
  return (
    <button
      type="button"
      className={`icp-option${active ? " icp-active" : ""}`}
      onMouseEnter={onHover}
      onClick={onChoose}
      disabled={disabled}
    >
      <span className="icp-option-main">
        <span className="icp-option-label">{label}</span>
        {option.recommended && <span className="icp-recommended">{recommendedLabel}</span>}
      </span>
      <span className="icp-option-description">{option.description}</span>
      {active && <ChevronRight className="icp-arrow" aria-hidden="true" />}
    </button>
  );
}
