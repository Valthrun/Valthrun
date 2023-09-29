# Driver entry should return 0x0
| Code | Description |
| :-- | :-- |
| 0xc0000603 | `STATUS_IMAGE_CERT_REVOKED` Most likely due to using the intel driver as vulnable driver (kdmapper issue [#65](https://github.com/TheCruZ/kdmapper/issues/65)). Disable [MSFT Driver Block List](./030_windows_security_features.md) |
| 0xCF000001 | The Valthrun logging system could not be initialized. This should only rarly occurr |
| 0xCF000002 | A function call, setting up the valthrun driver has failed. Lookup DebugView for more details. |
| 0xCF000003 | The valthrun driver failed to initialize. Lookup DebugView for more details. |
| 0xCF000004 | The valthrun driver has already been loaded | 
