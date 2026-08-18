export interface JobData {
  title: string;
  company: string;
  location: string;
  description: string;
  url: string;
  salary?: string;
  employmentType?: string;
  requirements?: string[];
  responsibilities?: string[];
}

export interface IpcRequest {
  protocolVersion: number;
  requestId: string;
  operation: string;
  payload: unknown;
}

export interface IpcResponse {
  protocolVersion: number;
  requestId: string;
  success: boolean;
  data?: unknown;
  error?: {
    code: string;
    message: string;
  };
}

export interface Adapter {
  canHandle(url: string): boolean;
  extractJob(): Promise<JobData | null>;
}
