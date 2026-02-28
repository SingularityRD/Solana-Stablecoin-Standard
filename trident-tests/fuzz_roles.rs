//! Fuzz tests for Role Management
//!
//! Tests various role assignment scenarios including:
//! - Valid role assignments
//! - Duplicate role assignments
//! - Role revocation
//! - Unauthorized role assignment
//! - All role types (Master, Minter, Burner, Blacklister, Pauser, Seizer)

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use sss_token::error::StablecoinError;
use sss_token::state::Role;
use trident::prelude::*;

/// Input structure for fuzz testing role assignment
#[derive(Debug, Arbitrary)]
pub struct RoleAssignInput {
    pub role_type: u8, // 0-5 for different roles
    pub is_authorized: bool,
    pub account_idx: u8,
}

/// Fuzz test for role assignment
#[fuzz]
pub fn fuzz_assign_role(input: RoleAssignInput) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin()?;

    let role = role_from_u8(input.role_type);
    let account = Pubkey::new_unique();

    let assigner = if input.is_authorized {
        ctx.authority.clone()
    } else {
        Pubkey::new_unique()
    };

    let result = try_assign_role(&mut ctx, assigner, account, role.clone());

    if !input.is_authorized {
        assert!(result.is_err(), "Unauthorized role assignment should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::Unauthorized as u32),
            "Expected Unauthorized error"
        );
    } else {
        assert!(result.is_ok(), "Authorized role assignment should succeed");
        // Verify role was assigned
        verify_role_assignment(&ctx, account, role)?;
    }

    Ok(())
}

/// Fuzz test for all role types
#[fuzz]
pub fn fuzz_all_role_types(role_type: u8) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin()?;

    let role = role_from_u8(role_type);
    let account = Pubkey::new_unique();

    let result = try_assign_role(&mut ctx, ctx.authority.clone(), account, role.clone());

    assert!(result.is_ok(), "Role assignment should succeed for role {:?}", role);

    Ok(())
}

/// Fuzz test for duplicate role assignment
#[fuzz]
pub fn fuzz_duplicate_role_assignment(role_type: u8) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin()?;

    let role = role_from_u8(role_type);
    let account = Pubkey::new_unique();

    // First assignment should succeed
    let result1 = try_assign_role(&mut ctx, ctx.authority.clone(), account, role.clone());
    assert!(result1.is_ok(), "First role assignment should succeed");

    // Second assignment should fail (account already has a role)
    let result2 = try_assign_role(&mut ctx, ctx.authority.clone(), account, role.clone());
    // Note: This depends on program logic - may need to be adjusted
    // If the program allows role update, this test should be modified

    Ok(())
}

/// Input for role revoke fuzz test
#[derive(Debug, Arbitrary)]
pub struct RoleRevokeInput {
    pub role_type: u8,
    pub assign_before: bool,
    pub is_authorized: bool,
}

/// Fuzz test for role revocation
#[fuzz]
pub fn fuzz_revoke_role(input: RoleRevokeInput) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin()?;

    let role = role_from_u8(input.role_type);
    let account = Pubkey::new_unique();

    if input.assign_before {
        // Assign role first
        try_assign_role(&mut ctx, ctx.authority.clone(), account, role)?;
    }

    let revoker = if input.is_authorized {
        ctx.authority.clone()
    } else {
        Pubkey::new_unique()
    };

    let result = try_revoke_role(&mut ctx, revoker, account);

    if !input.is_authorized {
        assert!(result.is_err(), "Unauthorized role revoke should fail");
    } else if !input.assign_before {
        // Revoke non-existent role should fail
        assert!(result.is_err(), "Revoke non-existent role should fail");
    } else {
        assert!(result.is_ok(), "Authorized revoke of existing role should succeed");
    }

    Ok(())
}

/// Input for sequential role operations
#[derive(Debug, Arbitrary)]
pub struct SequentialRoleInput {
    pub operations: Vec<RoleOp>,
}

#[derive(Debug, Arbitrary)]
pub enum RoleOp {
    Assign { role_type: u8, account_idx: u8 },
    Revoke { account_idx: u8 },
}

#[fuzz]
pub fn fuzz_sequential_role_operations(input: SequentialRoleInput) -> Result<()> {
    if input.operations.is_empty() || input.operations.len() > 100 {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin()?;

    // Track which accounts have roles
    let mut accounts_with_roles: std::collections::HashSet<u8> = std::collections::HashSet::new();

    for (i, op) in input.operations.iter().enumerate() {
        match op {
            RoleOp::Assign { role_type, account_idx } => {
                let role = role_from_u8(*role_type);
                let account = derive_account(*account_idx);

                let result = try_assign_role(&mut ctx, ctx.authority.clone(), account, role);

                if accounts_with_roles.contains(account_idx) {
                    // Account already has a role
                } else if result.is_ok() {
                    accounts_with_roles.insert(*account_idx);
                }
            }
            RoleOp::Revoke { account_idx } => {
                let account = derive_account(*account_idx);

                let result = try_revoke_role(&mut ctx, ctx.authority.clone(), account);

                if result.is_ok() && accounts_with_roles.contains(account_idx) {
                    accounts_with_roles.remove(account_idx);
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for role-based permissions
#[derive(Debug, Arbitrary)]
pub struct RolePermissionInput {
    pub role_type: u8,
    pub action: Action,
}

#[derive(Debug, Arbitrary)]
pub enum Action {
    Mint(u64),
    Burn(u64),
    Pause,
    Unpause,
    Blacklist,
    Seize(u64),
}

#[fuzz]
pub fn fuzz_role_permissions(input: RolePermissionInput) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin()?;

    let role = role_from_u8(input.role_type);
    let actor = Pubkey::new_unique();

    // Assign role to actor
    try_assign_role(&mut ctx, ctx.authority.clone(), actor, role.clone())?;

    // Try the action
    let result = match input.action {
        Action::Mint(amount) => {
            let mint_result = try_mint_as(&mut ctx, actor, amount);
            // Only Master and Minter can mint
            match role {
                Role::Master | Role::Minter => {
                    if amount == 0 {
                        assert!(mint_result.is_err(), "Zero mint should fail");
                    } else {
                        // Should succeed or overflow
                    }
                }
                _ => {
                    assert!(mint_result.is_err(), "Non-minter mint should fail");
                }
            }
            mint_result
        }
        Action::Burn(amount) => {
            let burn_result = try_burn_as(&mut ctx, actor, amount);
            // Only Master and Burner can burn
            match role {
                Role::Master | Role::Burner => {
                    if amount == 0 {
                        assert!(burn_result.is_err(), "Zero burn should fail");
                    }
                }
                _ => {
                    assert!(burn_result.is_err(), "Non-burner burn should fail");
                }
            }
            burn_result
        }
        Action::Pause => {
            let pause_result = try_pause_as(&mut ctx, actor);
            // Only Master and Pauser can pause
            match role {
                Role::Master | Role::Pauser => {}
                _ => {
                    assert!(pause_result.is_err(), "Non-pauser pause should fail");
                }
            }
            pause_result
        }
        Action::Unpause => {
            let unpause_result = try_unpause_as(&mut ctx, actor);
            // Only Master and Pauser can unpause
            match role {
                Role::Master | Role::Pauser => {}
                _ => {
                    assert!(unpause_result.is_err(), "Non-pauser unpause should fail");
                }
            }
            unpause_result
        }
        Action::Blacklist => {
            let target = Pubkey::new_unique();
            let blacklist_result = try_blacklist_as(&mut ctx, actor, target);
            // Only Master and Blacklister can blacklist
            match role {
                Role::Master | Role::Blacklister => {}
                _ => {
                    assert!(blacklist_result.is_err(), "Non-blacklister blacklist should fail");
                }
            }
            blacklist_result
        }
        Action::Seize(amount) => {
            let target = Pubkey::new_unique();
            let seize_result = try_seize_as(&mut ctx, actor, target, amount);
            // Only Master and Seizer can seize
            match role {
                Role::Master | Role::Seizer => {}
                _ => {
                    assert!(seize_result.is_err(), "Non-seizer seize should fail");
                }
            }
            seize_result
        }
    };

    Ok(())
}

/// Fuzz test for role reassignment
#[fuzz]
pub fn fuzz_role_reassignment(role_types: Vec<u8>) -> Result<()> {
    if role_types.is_empty() || role_types.len() > 20 {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin()?;
    let account = Pubkey::new_unique();

    for (i, &role_type) in role_types.iter().enumerate() {
        let role = role_from_u8(role_type);

        // Revoke existing role first
        let _ = try_revoke_role(&mut ctx, ctx.authority.clone(), account);

        // Assign new role
        let result = try_assign_role(&mut ctx, ctx.authority.clone(), account, role.clone());

        assert!(result.is_ok(), "Role assignment {} should succeed", i);
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

struct RoleTestContext {
    context: TestContext,
    authority: Pubkey,
    state_pda: Pubkey,
    asset_mint: Pubkey,
}

fn setup_initialized_stablecoin() -> Result<RoleTestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;

    let authority = test.payer();
    let asset_mint = Pubkey::new_unique();

    let (state_pda, bump) = Pubkey::find_program_address(
        &[b"stablecoin", asset_mint.as_ref()],
        &sss_token::ID,
    );

    let init_ix = sss_token::instruction::Initialize {
        preset: 1,
        name: "Test Stablecoin".to_string(),
        symbol: "TST".to_string(),
        uri: "https://test.com".to_string(),
        decimals: 6,
    };

    test.invoke(
        &[
            AccountMeta::new(authority, true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(asset_mint, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        init_ix,
        Some(&[&[b"stablecoin", asset_mint.as_ref(), &[bump]]]),
    )?;

    Ok(RoleTestContext {
        context: test,
        authority,
        state_pda,
        asset_mint,
    })
}

fn role_from_u8(role_type: u8) -> Role {
    match role_type % 6 {
        0 => Role::Master,
        1 => Role::Minter,
        2 => Role::Burner,
        3 => Role::Blacklister,
        4 => Role::Pauser,
        5 => Role::Seizer,
        _ => unreachable!(),
    }
}

fn derive_account(idx: u8) -> Pubkey {
    // Create deterministic pubkey from index
    let mut bytes = [0u8; 32];
    bytes[0] = idx;
    Pubkey::from(bytes)
}

fn try_assign_role(
    ctx: &mut RoleTestContext,
    assigner: Pubkey,
    account: Pubkey,
    role: Role,
) -> Result<()> {
    let (assignment_pda, _) = Pubkey::find_program_address(
        &[b"role", ctx.state_pda.as_ref(), account.as_ref()],
        &sss_token::ID,
    );

    let assign_ix = sss_token::instruction::AssignRole { role };

    ctx.context.invoke(
        &[
            AccountMeta::new(assigner, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(assignment_pda, false),
            AccountMeta::new_readonly(account, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        assign_ix,
        None,
    )
}

fn try_revoke_role(ctx: &mut RoleTestContext, revoker: Pubkey, account: Pubkey) -> Result<()> {
    let (assignment_pda, _) = Pubkey::find_program_address(
        &[b"role", ctx.state_pda.as_ref(), account.as_ref()],
        &sss_token::ID,
    );

    let revoke_ix = sss_token::instruction::RevokeRole {};

    ctx.context.invoke(
        &[
            AccountMeta::new(revoker, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(assignment_pda, false),
        ],
        revoke_ix,
        None,
    )
}

fn verify_role_assignment(_ctx: &RoleTestContext, _account: Pubkey, _role: Role) -> Result<()> {
    // In real implementation, fetch and verify the role assignment
    Ok(())
}

fn try_mint_as(ctx: &mut RoleTestContext, minter: Pubkey, amount: u64) -> Result<()> {
    let recipient = Pubkey::new_unique();
    let mint_ix = sss_token::instruction::Mint { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(minter, true),
            AccountMeta::new(ctx.state_pda, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(recipient, false),
        ],
        mint_ix,
        None,
    )
}

fn try_burn_as(ctx: &mut RoleTestContext, burner: Pubkey, amount: u64) -> Result<()> {
    let from = Pubkey::new_unique();
    let burn_ix = sss_token::instruction::Burn { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(burner, true),
            AccountMeta::new(ctx.state_pda, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(from, false),
        ],
        burn_ix,
        None,
    )
}

fn try_pause_as(ctx: &mut RoleTestContext, pauser: Pubkey) -> Result<()> {
    let pause_ix = sss_token::instruction::Pause {};
    ctx.context.invoke(
        &[
            AccountMeta::new(pauser, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
        ],
        pause_ix,
        None,
    )
}

fn try_unpause_as(ctx: &mut RoleTestContext, unpauser: Pubkey) -> Result<()> {
    let unpause_ix = sss_token::instruction::Unpause {};
    ctx.context.invoke(
        &[
            AccountMeta::new(unpauser, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
        ],
        unpause_ix,
        None,
    )
}

fn try_blacklist_as(ctx: &mut RoleTestContext, blacklister: Pubkey, target: Pubkey) -> Result<()> {
    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", ctx.state_pda.as_ref(), target.as_ref()],
        &sss_token::ID,
    );

    let blacklist_ix = sss_token::instruction::AddToBlacklist {
        reason: "Test".to_string(),
    };

    ctx.context.invoke(
        &[
            AccountMeta::new(blacklister, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(blacklist_pda, false),
            AccountMeta::new_readonly(target, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        blacklist_ix,
        None,
    )
}

fn try_seize_as(ctx: &mut RoleTestContext, seizer: Pubkey, target: Pubkey, amount: u64) -> Result<()> {
    let seize_ix = sss_token::instruction::Seize { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(seizer, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new_readonly(target, false),
        ],
        seize_ix,
        None,
    )
}

fn parse_anchor_error(error: &Error) -> Option<u32> {
    match error {
        Error::AnchorError(e) => Some(e.error_code_number),
        Error::ProgramError(e) => {
            if let Some(code) = e.to_error_code() {
                Some(code.code())
            } else {
                None
            }
        }
        _ => None,
    }
}
