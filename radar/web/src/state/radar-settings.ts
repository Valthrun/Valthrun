import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { kReduxPersistLocalStorage } from "./storage";
import { persistReducer } from "redux-persist";

export type State = {
    dialogOpen: boolean,

    iconSize: number,
};
const slice = createSlice({
    name: "radar-settings",
    initialState: (): State => ({
        dialogOpen: false,
        iconSize: 3.0
    }),
    reducers: {
        updateRadarSettings: (state, action: PayloadAction<Partial<State>>) => {
            Object.assign(state, action.payload);
        }
    }
});

export default persistReducer({
    key: "radar-settings",
    storage: kReduxPersistLocalStorage
}, slice.reducer);

export const {
    updateRadarSettings,
} = slice.actions;
