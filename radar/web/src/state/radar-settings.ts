import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { persistReducer } from "redux-persist";
import { kReduxPersistLocalStorage } from "./storage";

export type RadarSettingsState = {
    dialogOpen: boolean;

    iconSize: number;
    displayBombDetails: boolean;

    colorDotCT: string;
    colorDotT: string;
    colorDotOwn: string;
    showDotOwn: boolean;
};

export const kDefaultRadarSettings: RadarSettingsState = {
    dialogOpen: false,
    iconSize: 3.0,
    displayBombDetails: true,

    colorDotCT: "#0007ff",
    colorDotT: "#ffc933",
    colorDotOwn: "#e91e63",

    showDotOwn: true,
};
const slice = createSlice({
    name: "radar-settings",
    initialState: () => kDefaultRadarSettings,
    reducers: {
        updateRadarSettings: (state, action: PayloadAction<Partial<RadarSettingsState>>) => {
            Object.assign(state, action.payload);
        },
    },
});

export default persistReducer(
    {
        key: "radar-settings",
        storage: kReduxPersistLocalStorage,
    },
    slice.reducer,
);

export const { updateRadarSettings } = slice.actions;
