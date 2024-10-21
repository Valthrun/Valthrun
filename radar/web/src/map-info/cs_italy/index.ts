import { LoadedMap } from "..";
import OfficialDefault from "./radar_1_default.png";

export default {
    mapName: "cs_italy",
    displayName: "Italy",

    pos_x: -2647, // upper left world coordinate
    pos_y: 2592,
    scale: 4.6,

    verticalSections: {
        default: // use the primary radar image
        {
            altitudeMax: 10000,
            altitudeMin: -10000,
        },
    },

    mapImages: [
        {
            name: "Official",
            images:{
                default: OfficialDefault
            }
        }
    ]
} satisfies LoadedMap;
