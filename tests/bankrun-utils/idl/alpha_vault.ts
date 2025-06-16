/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/alpha_vault.json`.
 */
export type AlphaVault = {
  "address": "SNPmGgnywBvvrAKMLundzG6StojyHTHDLu7T4sdhP4k",
  "metadata": {
    "name": "alphaVault",
    "version": "0.4.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "claimToken",
      "discriminator": [
        116,
        206,
        27,
        191,
        166,
        19,
        0,
        73
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "escrow",
          "writable": true
        },
        {
          "name": "tokenOutVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "destinationToken",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "owner",
          "signer": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    },
    {
      "name": "closeCrankFeeWhitelist",
      "discriminator": [
        189,
        166,
        73,
        241,
        81,
        12,
        246,
        170
      ],
      "accounts": [
        {
          "name": "crankFeeWhitelist",
          "writable": true
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "rentReceiver",
          "writable": true
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    },
    {
      "name": "closeEscrow",
      "discriminator": [
        139,
        171,
        94,
        146,
        191,
        91,
        144,
        50
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "escrow",
          "writable": true
        },
        {
          "name": "owner",
          "signer": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "rentReceiver",
          "writable": true
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    },
    {
      "name": "closeFcfsConfig",
      "discriminator": [
        48,
        178,
        212,
        101,
        23,
        138,
        233,
        90
      ],
      "accounts": [
        {
          "name": "config",
          "writable": true
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "rentReceiver",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "closeProrataConfig",
      "discriminator": [
        84,
        140,
        103,
        57,
        178,
        155,
        57,
        26
      ],
      "accounts": [
        {
          "name": "config",
          "writable": true
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "rentReceiver",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "createCrankFeeWhitelist",
      "discriminator": [
        120,
        91,
        25,
        162,
        211,
        27,
        100,
        199
      ],
      "accounts": [
        {
          "name": "crankFeeWhitelist",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  97,
                  110,
                  107,
                  95,
                  102,
                  101,
                  101,
                  95,
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "cranker"
              }
            ]
          }
        },
        {
          "name": "cranker"
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    },
    {
      "name": "createFcfsConfig",
      "discriminator": [
        7,
        255,
        242,
        242,
        1,
        99,
        179,
        12
      ],
      "accounts": [
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  99,
                  102,
                  115,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "arg",
                "path": "config_parameters.index"
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "configParameters",
          "type": {
            "defined": {
              "name": "fcfsConfigParameters"
            }
          }
        }
      ]
    },
    {
      "name": "createMerkleRootConfig",
      "discriminator": [
        55,
        243,
        253,
        240,
        78,
        186,
        232,
        166
      ],
      "accounts": [
        {
          "name": "vault"
        },
        {
          "name": "merkleRootConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  107,
                  108,
                  101,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "arg",
                "path": "params.version"
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "createMerkleRootConfigParams"
            }
          }
        }
      ]
    },
    {
      "name": "createNewEscrow",
      "discriminator": [
        60,
        154,
        170,
        202,
        252,
        109,
        83,
        199
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "owner"
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "escrowFeeReceiver",
          "writable": true,
          "optional": true,
          "address": "BJQbRiRWhJCyTYZcAuAL3ngDCx3AyFQGKDq8zhiZAKUw"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    },
    {
      "name": "createPermissionedEscrow",
      "discriminator": [
        60,
        166,
        36,
        85,
        96,
        137,
        132,
        184
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "merkleRootConfig"
          ]
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "owner"
        },
        {
          "name": "merkleRootConfig",
          "docs": [
            "merkleRootConfig"
          ]
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "escrowFeeReceiver",
          "writable": true,
          "optional": true,
          "address": "BJQbRiRWhJCyTYZcAuAL3ngDCx3AyFQGKDq8zhiZAKUw"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxCap",
          "type": "u64"
        },
        {
          "name": "proof",
          "type": {
            "vec": {
              "array": [
                "u8",
                32
              ]
            }
          }
        }
      ]
    },
    {
      "name": "createPermissionedEscrowWithAuthority",
      "discriminator": [
        211,
        231,
        194,
        69,
        65,
        11,
        123,
        93
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "owner"
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxCap",
          "type": "u64"
        }
      ]
    },
    {
      "name": "createProrataConfig",
      "discriminator": [
        38,
        203,
        72,
        231,
        103,
        29,
        195,
        61
      ],
      "accounts": [
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  114,
                  97,
                  116,
                  97,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "arg",
                "path": "config_parameters.index"
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "configParameters",
          "type": {
            "defined": {
              "name": "prorataConfigParameters"
            }
          }
        }
      ]
    },
    {
      "name": "deposit",
      "discriminator": [
        242,
        35,
        198,
        137,
        82,
        225,
        242,
        182
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true
        },
        {
          "name": "sourceToken",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "owner",
          "signer": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "fillDammV2",
      "discriminator": [
        221,
        175,
        108,
        48,
        19,
        204,
        125,
        23
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenOutVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "ammProgram",
          "address": "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG"
        },
        {
          "name": "poolAuthority"
        },
        {
          "name": "pool",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenAVault",
          "writable": true
        },
        {
          "name": "tokenBVault",
          "writable": true
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        },
        {
          "name": "dammEventAuthority"
        },
        {
          "name": "crankFeeWhitelist",
          "optional": true
        },
        {
          "name": "crankFeeReceiver",
          "writable": true,
          "optional": true,
          "address": "BJQbRiRWhJCyTYZcAuAL3ngDCx3AyFQGKDq8zhiZAKUw"
        },
        {
          "name": "cranker",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "fillDlmm",
      "discriminator": [
        1,
        108,
        141,
        11,
        4,
        126,
        251,
        222
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenOutVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "ammProgram",
          "address": "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        },
        {
          "name": "pool",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "binArrayBitmapExtension"
        },
        {
          "name": "reserveX",
          "writable": true
        },
        {
          "name": "reserveY",
          "writable": true
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "oracle",
          "writable": true
        },
        {
          "name": "tokenXProgram"
        },
        {
          "name": "tokenYProgram"
        },
        {
          "name": "dlmmEventAuthority"
        },
        {
          "name": "crankFeeWhitelist",
          "optional": true
        },
        {
          "name": "crankFeeReceiver",
          "writable": true,
          "optional": true,
          "address": "BJQbRiRWhJCyTYZcAuAL3ngDCx3AyFQGKDq8zhiZAKUw"
        },
        {
          "name": "cranker",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "memoProgram"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxAmount",
          "type": "u64"
        },
        {
          "name": "remainingAccountsInfo",
          "type": {
            "defined": {
              "name": "remainingAccountsInfo"
            }
          }
        }
      ]
    },
    {
      "name": "fillDynamicAmm",
      "discriminator": [
        224,
        226,
        223,
        80,
        36,
        50,
        70,
        231
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenOutVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "ammProgram",
          "address": "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB"
        },
        {
          "name": "pool",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "aVault",
          "writable": true
        },
        {
          "name": "bVault",
          "writable": true
        },
        {
          "name": "aTokenVault",
          "writable": true
        },
        {
          "name": "bTokenVault",
          "writable": true
        },
        {
          "name": "aVaultLpMint",
          "writable": true
        },
        {
          "name": "bVaultLpMint",
          "writable": true
        },
        {
          "name": "aVaultLp",
          "writable": true
        },
        {
          "name": "bVaultLp",
          "writable": true
        },
        {
          "name": "adminTokenFee",
          "writable": true
        },
        {
          "name": "vaultProgram"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "crankFeeWhitelist",
          "optional": true
        },
        {
          "name": "crankFeeReceiver",
          "writable": true,
          "optional": true,
          "address": "BJQbRiRWhJCyTYZcAuAL3ngDCx3AyFQGKDq8zhiZAKUw"
        },
        {
          "name": "cranker",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "maxAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initializeFcfsVault",
      "discriminator": [
        163,
        205,
        69,
        145,
        235,
        71,
        47,
        21
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "base"
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "pool"
        },
        {
          "name": "funder",
          "writable": true,
          "signer": true
        },
        {
          "name": "base",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "initializeFcfsVaultParams"
            }
          }
        }
      ]
    },
    {
      "name": "initializeProrataVault",
      "discriminator": [
        178,
        180,
        176,
        247,
        128,
        186,
        43,
        9
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "base"
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "pool"
        },
        {
          "name": "funder",
          "writable": true,
          "signer": true
        },
        {
          "name": "base",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "initializeProrataVaultParams"
            }
          }
        }
      ]
    },
    {
      "name": "initializeVaultWithFcfsConfig",
      "discriminator": [
        189,
        251,
        92,
        104,
        235,
        21,
        81,
        182
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "pool"
        },
        {
          "name": "quoteMint"
        },
        {
          "name": "funder",
          "writable": true,
          "signer": true
        },
        {
          "name": "config"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "initializeVaultWithConfigParams"
            }
          }
        }
      ]
    },
    {
      "name": "initializeVaultWithProrataConfig",
      "discriminator": [
        155,
        216,
        34,
        162,
        103,
        242,
        236,
        211
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "pool"
        },
        {
          "name": "quoteMint"
        },
        {
          "name": "funder",
          "writable": true,
          "signer": true
        },
        {
          "name": "config"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "initializeVaultWithConfigParams"
            }
          }
        }
      ]
    },
    {
      "name": "transferVaultAuthority",
      "discriminator": [
        139,
        35,
        83,
        88,
        52,
        186,
        162,
        110
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "vaultAuthority",
          "signer": true,
          "relations": [
            "vault"
          ]
        }
      ],
      "args": [
        {
          "name": "newAuthority",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "updateFcfsVaultParameters",
      "discriminator": [
        172,
        23,
        13,
        143,
        18,
        133,
        104,
        174
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "admin",
          "signer": true
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "updateFcfsVaultParams"
            }
          }
        }
      ]
    },
    {
      "name": "updateProrataVaultParameters",
      "discriminator": [
        177,
        39,
        151,
        50,
        253,
        249,
        5,
        74
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "admin",
          "signer": true
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "updateProrataVaultParams"
            }
          }
        }
      ]
    },
    {
      "name": "withdraw",
      "discriminator": [
        183,
        18,
        70,
        156,
        148,
        109,
        161,
        34
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true
        },
        {
          "name": "destinationToken",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "owner",
          "signer": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdrawRemainingQuote",
      "discriminator": [
        54,
        253,
        188,
        34,
        100,
        145,
        59,
        127
      ],
      "accounts": [
        {
          "name": "vault",
          "writable": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "pool",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "escrow",
          "writable": true
        },
        {
          "name": "tokenVault",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "destinationToken",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "owner",
          "signer": true,
          "relations": [
            "escrow"
          ]
        },
        {
          "name": "eventAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  95,
                  95,
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "program"
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "crankFeeWhitelist",
      "discriminator": [
        39,
        105,
        184,
        30,
        248,
        231,
        176,
        133
      ]
    },
    {
      "name": "escrow",
      "discriminator": [
        31,
        213,
        123,
        187,
        186,
        22,
        218,
        155
      ]
    },
    {
      "name": "fcfsVaultConfig",
      "discriminator": [
        99,
        243,
        252,
        122,
        160,
        175,
        130,
        52
      ]
    },
    {
      "name": "merkleRootConfig",
      "discriminator": [
        103,
        2,
        222,
        217,
        73,
        50,
        187,
        39
      ]
    },
    {
      "name": "prorataVaultConfig",
      "discriminator": [
        93,
        214,
        205,
        104,
        119,
        9,
        51,
        152
      ]
    },
    {
      "name": "vault",
      "discriminator": [
        211,
        8,
        232,
        43,
        2,
        152,
        117,
        119
      ]
    }
  ],
  "events": [
    {
      "name": "crankFeeWhitelistClosed",
      "discriminator": [
        157,
        171,
        85,
        155,
        37,
        20,
        41,
        114
      ]
    },
    {
      "name": "crankFeeWhitelistCreated",
      "discriminator": [
        176,
        138,
        32,
        77,
        129,
        74,
        137,
        244
      ]
    },
    {
      "name": "escrowClaimToken",
      "discriminator": [
        179,
        72,
        71,
        30,
        59,
        19,
        170,
        3
      ]
    },
    {
      "name": "escrowClosed",
      "discriminator": [
        109,
        20,
        57,
        51,
        217,
        118,
        3,
        173
      ]
    },
    {
      "name": "escrowCreated",
      "discriminator": [
        70,
        127,
        105,
        102,
        92,
        97,
        7,
        173
      ]
    },
    {
      "name": "escrowDeposit",
      "discriminator": [
        43,
        90,
        49,
        176,
        134,
        148,
        50,
        32
      ]
    },
    {
      "name": "escrowRemainingWithdraw",
      "discriminator": [
        113,
        14,
        156,
        89,
        113,
        79,
        88,
        178
      ]
    },
    {
      "name": "escrowWithdraw",
      "discriminator": [
        171,
        17,
        164,
        116,
        122,
        66,
        183,
        34
      ]
    },
    {
      "name": "fcfsVaultCreated",
      "discriminator": [
        73,
        153,
        165,
        103,
        151,
        182,
        184,
        136
      ]
    },
    {
      "name": "fcfsVaultParametersUpdated",
      "discriminator": [
        78,
        112,
        112,
        62,
        193,
        209,
        231,
        226
      ]
    },
    {
      "name": "merkleRootConfigCreated",
      "discriminator": [
        121,
        112,
        42,
        76,
        144,
        131,
        142,
        90
      ]
    },
    {
      "name": "prorataVaultCreated",
      "discriminator": [
        181,
        255,
        162,
        226,
        203,
        199,
        193,
        6
      ]
    },
    {
      "name": "prorataVaultParametersUpdated",
      "discriminator": [
        24,
        147,
        160,
        237,
        132,
        87,
        15,
        206
      ]
    },
    {
      "name": "swapFill",
      "discriminator": [
        116,
        212,
        73,
        222,
        33,
        244,
        134,
        148
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "timePointNotInFuture",
      "msg": "Time point is not in future"
    },
    {
      "code": 6001,
      "name": "incorrectTokenMint",
      "msg": "Token mint is incorrect"
    },
    {
      "code": 6002,
      "name": "incorrectPairType",
      "msg": "Pair is not permissioned"
    },
    {
      "code": 6003,
      "name": "poolHasStarted",
      "msg": "Pool has started"
    },
    {
      "code": 6004,
      "name": "notPermitThisActionInThisTimePoint",
      "msg": "This action is not permitted in this time point"
    },
    {
      "code": 6005,
      "name": "theSaleIsOngoing",
      "msg": "The sale is on going, cannot withdraw"
    },
    {
      "code": 6006,
      "name": "escrowIsNotClosable",
      "msg": "Escrow is not closable"
    },
    {
      "code": 6007,
      "name": "timePointOrdersAreIncorrect",
      "msg": "Time point orders are incorrect"
    },
    {
      "code": 6008,
      "name": "escrowHasRefuned",
      "msg": "Escrow has refunded"
    },
    {
      "code": 6009,
      "name": "mathOverflow",
      "msg": "Math operation overflow"
    },
    {
      "code": 6010,
      "name": "maxBuyingCapIsZero",
      "msg": "Max buying cap is zero"
    },
    {
      "code": 6011,
      "name": "maxAmountIsTooSmall",
      "msg": "Max amount is too small"
    },
    {
      "code": 6012,
      "name": "poolTypeIsNotSupported",
      "msg": "Pool type is not supported"
    },
    {
      "code": 6013,
      "name": "invalidAdmin",
      "msg": "Invalid admin"
    },
    {
      "code": 6014,
      "name": "vaultModeIsIncorrect",
      "msg": "Vault mode is incorrect"
    },
    {
      "code": 6015,
      "name": "maxDepositingCapIsInValid",
      "msg": "Max depositing cap is invalid"
    },
    {
      "code": 6016,
      "name": "vestingDurationIsInValid",
      "msg": "Vesting duration is invalid"
    },
    {
      "code": 6017,
      "name": "depositAmountIsZero",
      "msg": "Deposit amount is zero"
    },
    {
      "code": 6018,
      "name": "poolOwnerIsMismatched",
      "msg": "Pool owner is mismatched"
    },
    {
      "code": 6019,
      "name": "refundAmountIsZero",
      "msg": "Refund amount is zero"
    },
    {
      "code": 6020,
      "name": "depositingDurationIsInvalid",
      "msg": "Depositing duration is invalid"
    },
    {
      "code": 6021,
      "name": "depositingTimePointIsInvalid",
      "msg": "Depositing time point is invalid"
    },
    {
      "code": 6022,
      "name": "individualDepositingCapIsZero",
      "msg": "Individual depositing cap is zero"
    },
    {
      "code": 6023,
      "name": "invalidFeeReceiverAccount",
      "msg": "Invalid fee receiver account"
    },
    {
      "code": 6024,
      "name": "notPermissionedVault",
      "msg": "Not permissioned vault"
    },
    {
      "code": 6025,
      "name": "notPermitToDoThisAction",
      "msg": "Not permit to do this action"
    },
    {
      "code": 6026,
      "name": "invalidProof",
      "msg": "Invalid Merkle proof"
    },
    {
      "code": 6027,
      "name": "invalidActivationType",
      "msg": "Invalid activation type"
    },
    {
      "code": 6028,
      "name": "activationTypeIsMismatched",
      "msg": "Activation type is mismatched"
    },
    {
      "code": 6029,
      "name": "invalidPool",
      "msg": "Pool is not connected to the alpha vault"
    },
    {
      "code": 6030,
      "name": "invalidCreator",
      "msg": "Invalid creator"
    },
    {
      "code": 6031,
      "name": "permissionedVaultCannotChargeEscrowFee",
      "msg": "Permissioned vault cannot charge escrow fee"
    },
    {
      "code": 6032,
      "name": "escrowFeeTooHigh",
      "msg": "Escrow fee too high"
    },
    {
      "code": 6033,
      "name": "lockDurationInvalid",
      "msg": "Lock duration is invalid"
    },
    {
      "code": 6034,
      "name": "maxBuyingCapIsTooSmall",
      "msg": "Max buying cap is too small"
    },
    {
      "code": 6035,
      "name": "maxDepositingCapIsTooSmall",
      "msg": "Max depositing cap is too small"
    },
    {
      "code": 6036,
      "name": "invalidWhitelistWalletMode",
      "msg": "Invalid whitelist wallet mode"
    },
    {
      "code": 6037,
      "name": "invalidCrankFeeWhitelist",
      "msg": "Invalid crank fee whitelist"
    },
    {
      "code": 6038,
      "name": "missingFeeReceiver",
      "msg": "Missing fee receiver"
    },
    {
      "code": 6039,
      "name": "discriminatorIsMismatched",
      "msg": "Discriminator is mismatched"
    }
  ],
  "types": [
    {
      "name": "accountsType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "transferHookX"
          },
          {
            "name": "transferHookY"
          },
          {
            "name": "transferHookReward"
          }
        ]
      }
    },
    {
      "name": "crankFeeWhitelist",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    },
    {
      "name": "crankFeeWhitelistClosed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "cranker",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "crankFeeWhitelistCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "cranker",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "createMerkleRootConfigParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "root",
            "docs": [
              "The 256-bit merkle root."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "version",
            "docs": [
              "version"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrow",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "docs": [
              "vault address"
            ],
            "type": "pubkey"
          },
          {
            "name": "owner",
            "docs": [
              "owner"
            ],
            "type": "pubkey"
          },
          {
            "name": "totalDeposit",
            "docs": [
              "total deposited quote token"
            ],
            "type": "u64"
          },
          {
            "name": "claimedToken",
            "docs": [
              "Total token that escrow has claimed"
            ],
            "type": "u64"
          },
          {
            "name": "lastClaimedPoint",
            "docs": [
              "Last claimed timestamp"
            ],
            "type": "u64"
          },
          {
            "name": "refunded",
            "docs": [
              "Whether owner has claimed for remaining quote token"
            ],
            "type": "u8"
          },
          {
            "name": "padding1",
            "docs": [
              "padding 1"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "maxCap",
            "docs": [
              "Only has meaning in permissioned vault"
            ],
            "type": "u64"
          },
          {
            "name": "padding2",
            "docs": [
              "padding 2"
            ],
            "type": {
              "array": [
                "u8",
                8
              ]
            }
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u128",
                1
              ]
            }
          }
        ]
      }
    },
    {
      "name": "escrowClaimToken",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "vaultTotalClaimedToken",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowClosed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "vaultTotalEscrow",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "vaultTotalEscrow",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowDeposit",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "vaultTotalDeposit",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowRemainingWithdraw",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "vaultRemainingDeposit",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowWithdraw",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "vaultTotalDeposit",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "fcfsConfigParameters",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "startVestingDuration",
            "type": "u64"
          },
          {
            "name": "endVestingDuration",
            "type": "u64"
          },
          {
            "name": "depositingDurationUntilLastJoinPoint",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          },
          {
            "name": "index",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "fcfsVaultConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "startVestingDuration",
            "type": "u64"
          },
          {
            "name": "endVestingDuration",
            "type": "u64"
          },
          {
            "name": "depositingDurationUntilLastJoinPoint",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                175
              ]
            }
          }
        ]
      }
    },
    {
      "name": "fcfsVaultCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "baseMint",
            "type": "pubkey"
          },
          {
            "name": "quoteMint",
            "type": "pubkey"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          },
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "poolType",
            "type": "u8"
          },
          {
            "name": "depositingPoint",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "fcfsVaultParametersUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          },
          {
            "name": "depositingPoint",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "initializeFcfsVaultParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "poolType",
            "type": "u8"
          },
          {
            "name": "quoteMint",
            "type": "pubkey"
          },
          {
            "name": "baseMint",
            "type": "pubkey"
          },
          {
            "name": "depositingPoint",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          },
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "whitelistMode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "initializeProrataVaultParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "poolType",
            "type": "u8"
          },
          {
            "name": "quoteMint",
            "type": "pubkey"
          },
          {
            "name": "baseMint",
            "type": "pubkey"
          },
          {
            "name": "depositingPoint",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          },
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "whitelistMode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "initializeVaultWithConfigParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "poolType",
            "type": "u8"
          },
          {
            "name": "quoteMint",
            "type": "pubkey"
          },
          {
            "name": "baseMint",
            "type": "pubkey"
          },
          {
            "name": "whitelistMode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "merkleRootConfig",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "root",
            "docs": [
              "The 256-bit merkle root."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "vault",
            "docs": [
              "vault pubkey that config is belong"
            ],
            "type": "pubkey"
          },
          {
            "name": "version",
            "docs": [
              "version"
            ],
            "type": "u64"
          },
          {
            "name": "padding",
            "docs": [
              "padding for further use"
            ],
            "type": {
              "array": [
                "u64",
                8
              ]
            }
          }
        ]
      }
    },
    {
      "name": "merkleRootConfigCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "version",
            "type": "u64"
          },
          {
            "name": "root",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "prorataConfigParameters",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "startVestingDuration",
            "type": "u64"
          },
          {
            "name": "endVestingDuration",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          },
          {
            "name": "index",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "prorataVaultConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "startVestingDuration",
            "type": "u64"
          },
          {
            "name": "endVestingDuration",
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u8",
                191
              ]
            }
          }
        ]
      }
    },
    {
      "name": "prorataVaultCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "baseMint",
            "type": "pubkey"
          },
          {
            "name": "quoteMint",
            "type": "pubkey"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          },
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "poolType",
            "type": "u8"
          },
          {
            "name": "escrowFee",
            "type": "u64"
          },
          {
            "name": "activationType",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "prorataVaultParametersUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "remainingAccountsInfo",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "slices",
            "type": {
              "vec": {
                "defined": {
                  "name": "remainingAccountsSlice"
                }
              }
            }
          }
        ]
      }
    },
    {
      "name": "remainingAccountsSlice",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "accountsType",
            "type": {
              "defined": {
                "name": "accountsType"
              }
            }
          },
          {
            "name": "length",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "swapFill",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "pair",
            "type": "pubkey"
          },
          {
            "name": "fillAmount",
            "type": "u64"
          },
          {
            "name": "purchasedAmount",
            "type": "u64"
          },
          {
            "name": "unfilledAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "updateFcfsVaultParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxDepositingCap",
            "type": "u64"
          },
          {
            "name": "depositingPoint",
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "updateProrataVaultParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxBuyingCap",
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "vault",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "docs": [
              "pool"
            ],
            "type": "pubkey"
          },
          {
            "name": "tokenVault",
            "docs": [
              "reserve quote token"
            ],
            "type": "pubkey"
          },
          {
            "name": "tokenOutVault",
            "docs": [
              "reserve base token"
            ],
            "type": "pubkey"
          },
          {
            "name": "quoteMint",
            "docs": [
              "quote token"
            ],
            "type": "pubkey"
          },
          {
            "name": "baseMint",
            "docs": [
              "base token"
            ],
            "type": "pubkey"
          },
          {
            "name": "base",
            "docs": [
              "base key"
            ],
            "type": "pubkey"
          },
          {
            "name": "owner",
            "docs": [
              "owner key, deprecated field, can re-use in the future"
            ],
            "type": "pubkey"
          },
          {
            "name": "maxBuyingCap",
            "docs": [
              "max buying cap"
            ],
            "type": "u64"
          },
          {
            "name": "totalDeposit",
            "docs": [
              "total deposited quote token"
            ],
            "type": "u64"
          },
          {
            "name": "totalEscrow",
            "docs": [
              "total user deposit"
            ],
            "type": "u64"
          },
          {
            "name": "swappedAmount",
            "docs": [
              "swappedAmount"
            ],
            "type": "u64"
          },
          {
            "name": "boughtToken",
            "docs": [
              "total bought token"
            ],
            "type": "u64"
          },
          {
            "name": "totalRefund",
            "docs": [
              "Total quote refund"
            ],
            "type": "u64"
          },
          {
            "name": "totalClaimedToken",
            "docs": [
              "Total claimed_token"
            ],
            "type": "u64"
          },
          {
            "name": "startVestingPoint",
            "docs": [
              "Start vesting ts"
            ],
            "type": "u64"
          },
          {
            "name": "endVestingPoint",
            "docs": [
              "End vesting ts"
            ],
            "type": "u64"
          },
          {
            "name": "bump",
            "docs": [
              "bump"
            ],
            "type": "u8"
          },
          {
            "name": "poolType",
            "docs": [
              "pool type"
            ],
            "type": "u8"
          },
          {
            "name": "vaultMode",
            "docs": [
              "vault mode"
            ],
            "type": "u8"
          },
          {
            "name": "padding0",
            "docs": [
              "padding 0"
            ],
            "type": {
              "array": [
                "u8",
                5
              ]
            }
          },
          {
            "name": "maxDepositingCap",
            "docs": [
              "max depositing cap"
            ],
            "type": "u64"
          },
          {
            "name": "individualDepositingCap",
            "docs": [
              "individual depositing cap"
            ],
            "type": "u64"
          },
          {
            "name": "depositingPoint",
            "docs": [
              "depositing point"
            ],
            "type": "u64"
          },
          {
            "name": "escrowFee",
            "docs": [
              "flat fee when user open an escrow"
            ],
            "type": "u64"
          },
          {
            "name": "totalEscrowFee",
            "docs": [
              "total escrow fee just for statistic"
            ],
            "type": "u64"
          },
          {
            "name": "whitelistMode",
            "docs": [
              "deposit whitelist mode"
            ],
            "type": "u8"
          },
          {
            "name": "activationType",
            "docs": [
              "activation type"
            ],
            "type": "u8"
          },
          {
            "name": "padding1",
            "docs": [
              "padding 1"
            ],
            "type": {
              "array": [
                "u8",
                6
              ]
            }
          },
          {
            "name": "vaultAuthority",
            "docs": [
              "vault authority normally is vault creator, will be able to create merkle root config"
            ],
            "type": "pubkey"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    }
  ]
};
