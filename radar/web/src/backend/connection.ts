import { EventEmitter } from "../utils/ee";


export type SubscriberClientState = {
    state: "new" | "connecting" | "initializing" | "connected" | "disconnected",
} | {
    state: "failed",
    reason: string
};

export interface SubscriberClientEvents {
    "state_changed": SubscriberClientState,
    "radar.state": RadarState,
}

export class SubscriberClient {
    readonly events: EventEmitter<SubscriberClientEvents>;

    private currentState: SubscriberClientState;
    private connection: WebSocket | null;

    private commandHandler: { [T in keyof S2CMessage]?: (payload: S2CMessage[T]) => void } = {};

    constructor(
        readonly targetAddress: string,
    ) {
        this.events = new EventEmitter();
        this.currentState = { state: "new" };
        this.connection = null;

        this.commandHandler = {};
        this.commandHandler["ResponseError"] = payload => {
            this.updateState({ state: "failed", reason: payload.error });
            this.closeSocket();
        };

        this.commandHandler["ResponseSessionInvalidId"] = () => {
            this.updateState({ state: "failed", reason: "session does not exists" });
            this.closeSocket();
        };

        this.commandHandler["ResponseSubscribeSuccess"] = () => {
            this.updateState({ state: "connected" });
        };

        this.commandHandler["NotifyRadarUpdate"] = payload => {
            this.events.emit("radar.state", payload.update.State.state)
        };

        this.commandHandler["NotifySessionClosed"] = () => {
            this.updateState({ state: "disconnected" });
        };
    }

    public getState(): Readonly<SubscriberClientState> {
        return this.currentState;
    }

    private updateState(newState: SubscriberClientState) {
        if (this.currentState === newState) {
            return;
        }

        this.currentState = newState;
        this.events.emit("state_changed", newState as any);
    }

    private closeSocket() {
        if (!this.connection) {
            return;
        }

        this.connection.onopen = undefined;
        this.connection.onclose = undefined;
        this.connection.onerror = undefined;
        this.connection.onmessage = undefined;
        if (this.connection.readyState === WebSocket.OPEN) {
            this.connection.close();
        }
        this.connection = null;
    }

    public connect(sessionId: string) {
        if (this.currentState.state != "new") {
            throw new Error(`invalid session state`);
        }

        this.updateState({ state: "connecting" });
        this.connection = new WebSocket(this.targetAddress);
        this.connection.onopen = () => {
            this.updateState({ state: "initializing" });
            this.sendCommand("InitializeSubscribe", {
                version: 1,
                session_id: sessionId
            });
        };

        this.connection.onerror = () => {
            this.updateState({ state: "failed", reason: "web socket error" });
            this.closeSocket();
        };

        this.connection.onclose = () => {
            if (this.currentState.state !== "disconnected") {
                this.updateState({ state: "failed", reason: "web socket closed" });
                this.closeSocket();
            }
        };

        this.connection.onmessage = event => {
            let payload = JSON.parse(event.data as string) as S2CMessage;
            if (typeof payload === "string") {
                payload = { [payload]: null } as any;
            }

            for (const key of Object.keys(payload)) {
                const commandHandler = this.commandHandler[key as any as keyof S2CMessage];
                if (typeof commandHandler === "function") {
                    commandHandler(payload[key as keyof typeof payload] as any);
                }
            }
        };
    }

    public sendCommand<T extends keyof C2SMessage>(command: T, payload: C2SMessage[T]) {
        this.connection.send(JSON.stringify({
            [command]: payload
        }));
    }
}

export type C2SMessage = {
    "InitializeSubscribe": { version: number, session_id: string },
}

export type S2CMessage = {
    "ResponseSuccess": void,
    "ResponseError": { error: string },
    "ResponseInvalidClientState": void,
    "ResponseInitializePublish": { session_id: string, version: number },
    "ResponseSubscribeSuccess": void,
    "ResponseSessionInvalidId": void,

    "NotifyRadarUpdate": {
        update: RadarUpdate
    },
    "NotifySessionClosed": void
}


export type RadarUpdate = {
    "State": { state: RadarState },
    /* "Settings": any */
};

export type RadarState = {
    players: RadarPlayerInfo[],
    worldName: string,
    bomb: RadarBombInfo,
};

export type RadarPlayerInfo = {
    controllerEntityId: number,
    teamId: number,

    playerHealth: number,
    playerHasDefuser: boolean,
    playerName: string,
    playerFlashtime: number,

    weapon: number,

    position: [number, number, number],
    rotation: number,
};

export type RadarBombInfo = {
    position: [number, number, number],
    state: C4State,
    bombSite: number | null,
};

export type C4State =
    | { variant: 'Carried' }
    | { variant: 'Dropped'}
    | {
    variant: 'Active';
    timeDetonation: number;
    defuse: BombDefuser | null;
}
    | { variant: 'Detonated' }
    | { variant: 'Defused' };

export type BombDefuser = {
    timeRemaining: number;
    playerName: string
};
