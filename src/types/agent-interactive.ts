export interface AgentInteractiveOption {
  label: string;
  description: string;
  recommended?: boolean;
  preview?: string;
}

export interface AgentInteractiveQuestion {
  header: string;
  question: string;
  options: AgentInteractiveOption[];
  multiSelect?: boolean;
}

export interface AgentInteractiveChoiceRequest {
  id: string;
  questions: AgentInteractiveQuestion[];
  currentIndex: number;
  total: number;
}

export interface AgentInteractiveAnswer {
  questionIndex: number;
  selectedLabels: string[];
  customAnswer?: string;
}
