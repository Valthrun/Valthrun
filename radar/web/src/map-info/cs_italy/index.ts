import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "cs_italy",
    displayName: "Italy",

    pos_x: -2647, // upper left world coordinate
    pos_y: 2592,
    scale: 4.6,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault
            }
        }
    ]
} satisfies LoadedMap;
