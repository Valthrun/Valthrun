// *** DO NOT EDIT ***"
// This file has been automatically generated.
// Invoke ts_gen in radar/shared to regenerate this file
export type U32 = number;
export type U8 = number;
export type I32 = number;
export type U16 = number;
export type F32 = number;
export type RadarPlayerInfo = {
    "controllerEntityId": U32;
    "pawnEntityId": U32;
    "teamId": U8;
    "playerHealth": I32;
    "playerHasDefuser": boolean;
    "playerName": string;
    "weapon": U16;
    "playerFlashtime": F32;
    "position": [F32, F32, F32];
    "rotation": F32;
};
export type BombDefuser = {

    /**
     * Total time remaining for a successful bomb defuse
     */
    "timeRemaining": F32;

    /**
     * Total time (in seconds) for the defusal
     */
    "timeTotal": F32;

    /**
     * The defusers player name
     */
    "playerName": string;
};
export type PlantedC4State = (({

    /**
     * Bomb is currently actively ticking
     */
    "state": "active";
} & {

    /**
     * Time remaining (in seconds) until detonation
     */
    "timeDetonation": F32;

    /**
     * Total time (in seconds) for the detonation
     */
    "timeTotal": F32;

    /**
     * Current bomb defuser
     */
    "defuser": (BombDefuser | null);
}) | ({

    /**
     * Bomb has detonated
     */
    "state": "detonated";
} & {
}) | ({

    /**
     * Bomb has been defused
     */
    "state": "defused";
} & {
}));
export type RadarPlantedC4 = {
    "position": [F32, F32, F32];

    /**
     * Planted bomb site
     * 0 = A
     * 1 = B
     */
    "bombSite": U8;
    "state": PlantedC4State;
};
export type RadarC4 = {
    "entityId": U32;
    "position": [F32, F32, F32];
    "ownerEntityId": (U32 | null);
};
export type RadarState = {
    "worldName": string;
    "players": (RadarPlayerInfo)[];
    "plantedC4": (RadarPlantedC4 | null);
    "c4Entities": (RadarC4)[];
};
export type Usize = number;
export type S2CMessage = ({
    "type": "response-success";
    "payload": {
    };
} | {
    "type": "response-error";
    "payload": {
        "error": string;
    };
} | {
    "type": "response-invalid-client-state";
    "payload": {
    };
} | {
    "type": "response-initialize-publish";
    "payload": {
        "session_id": string;
        "version": U32;
    };
} | {
    "type": "response-subscribe-success";
    "payload": {
    };
} | {
    "type": "response-session-invalid-id";
    "payload": {
    };
} | {
    "type": "notify-radar-state";
    "payload": {
        "state": RadarState;
    };
} | {
    "type": "notify-view-count";
    "payload": {
        "viewers": Usize;
    };
} | {
    "type": "notify-session-closed";
    "payload": {
    };
});
export type C2SMessage = ({
    "type": "initialize-publish";
    "payload": {
        "version": U32;
    };
} | {
    "type": "initialize-subscribe";
    "payload": {
        "version": U32;
        "session_id": string;
    };
} | {
    "type": "notify-radar-state";
    "payload": {
        "state": RadarState;
    };
} | {
    "type": "disconnect";
    "payload": {
        "reason": string;
    };
});
