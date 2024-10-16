import { configureStore } from "@reduxjs/toolkit";
import { TypedUseSelectorHook, useDispatch, useSelector } from "react-redux";
import {
    persistStore,
    FLUSH as ReduxPersistFlush,
    PAUSE as ReduxPersistPause,
    PERSIST as ReduxPersistPersist,
    PURGE as ReduxPersistPurge,
    REGISTER as ReduxPersistRegister,
    REHYDRATE as ReduxPersistRehydrate,
} from "redux-persist";
import kReducerRadarSettings from "./radar-settings";

export const appStore = configureStore({
    reducer: {
        radarSettings: kReducerRadarSettings,
    },
    middleware: (getDefaultMiddleware) => {
        return getDefaultMiddleware({
            serializableCheck: {
                ignoredActions: [
                    ReduxPersistFlush,
                    ReduxPersistRehydrate,
                    ReduxPersistPause,
                    ReduxPersistPersist,
                    ReduxPersistPurge,
                    ReduxPersistRegister,
                ],
            },
        });
    },
});

export async function initializeAppStore() {
    await new Promise<void>((resolve) => persistStore(appStore, null, resolve));
}

export type RootState = ReturnType<typeof appStore.getState>;
export type AppDispatch = typeof appStore.dispatch;

export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
