// Type Definitions

export interface ToolCall {
    tool: string;
    args: any;
    result: string;
}

export interface RunResult {
    output: string;
    turns: number;
    tool_calls: ToolCall[];
}

export interface Profession {
    name: string;
    model: string;
    temperature: number;
    max_turns: number;
}

export interface WorkflowResult {
    steps: any;
    outputs: any;
}

export interface UserInfo {
    username: string;
    role: string;
}

export interface LoginResult {
    token: string;
    user: UserInfo;
}

// API Functions

export async function run_agent(task: string, profession: string): Promise<RunResult> {
    const response = await fetch('/api/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ task, profession }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function list_professions(): Promise<any> {
    const response = await fetch('/api/professions', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function get_config(): Promise<any> {
    const response = await fetch('/api/config', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function list_skills(): Promise<any> {
    const response = await fetch('/api/skills', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function list_modes(): Promise<any> {
    const response = await fetch('/api/modes', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function list_workflows(): Promise<any> {
    const response = await fetch('/api/workflows', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function run_workflow(task: string, workflow: string): Promise<WorkflowResult> {
    const response = await fetch('/api/workflow/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ task, workflow }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function login(username: string, password: string): Promise<LoginResult> {
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function me(): Promise<UserInfo> {
    const response = await fetch('/api/auth/me', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function get_specs(): Promise<any> {
    const response = await fetch('/api/specs', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function upsert_spec(section_id: string, item: any): Promise<any> {
    const response = await fetch('/api/specs/item', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ section_id, item }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function transition_spec(section_id: string, item_id: string, new_status: string): Promise<any> {
    const response = await fetch('/api/specs/transition', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ section_id, item_id, new_status }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
