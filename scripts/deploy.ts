import { ContractPromise } from '@polkadot/api-contract'
import { deployContract, contractTx, decodeOutput, contractQuery } from '@scio-labs/use-inkathon'
import * as dotenv from 'dotenv'
import { getDeploymentData } from './utils/getDeploymentData'
import { initPolkadotJs } from './utils/initPolkadotJs'
import { writeContractAddresses } from './utils/writeContractAddresses'

// Dynamic environment variables
const chainId = process.env.CHAIN || 'development'
dotenv.config({
  path: `.env.${chainId}`,
})

/**
 * Deploys and configures contracts
 */
const main = async (pool_ids: number[]) => {
  // Initialization
  const initParams = await initPolkadotJs()
  const { api, chain, account } = initParams

  console.log('===== Contract Deployment =====')

  console.log(`Deploying contract: 'registry' ...`)
  const registry_data = await getDeploymentData('registry')
  const registry = await deployContract(
    api,
    account,
    registry_data.abi,
    registry_data.wasm,
    'new',
    [account.address, account.address, account.address],
  )

  console.log(`Deploying contract: 'share_token' ...`)
  const token_data = await getDeploymentData('share_token')
  const share_token = await deployContract(
    api,
    account,
    token_data.abi,
    token_data.wasm,
    'new',
    ['TEST', 'TS'],
  )

  console.log(`Deploying contract: 'vault' ...`)
  const vault_data = await getDeploymentData('vault')
  const vault = chainId === 'development'
    ? await deployContract(
      api,
      account,
      vault_data.abi,
      vault_data.wasm,
      'custom_era',
      [token_data.abi.source.hash, registry_data.abi.source.hash, 30 * 3 * 1_000],
    ) : await deployContract(
      api,
      account,
      vault_data.abi,
      vault_data.wasm,
      'new',
      [token_data.abi.source.hash, registry_data.abi.source.hash],
    )

  // Deploy agents for testing
  const nomination_agents = []
  for (const pool_id of pool_ids) {
    const nominator_data = await getDeploymentData('nomination_agent')
    console.log(`Deploying contract: 'nomination_agent' (PID #${pool_id}) ...`)
    const nominator = await deployContract(
      api,
      account,
      nominator_data.abi,
      nominator_data.wasm,
      'new',
      [vault.address, account.address, pool_id],
    )
    console.log('Deployed!')
    nomination_agents.push(nominator)
  }

  console.log('===== Contract Configuration =====')

  const vault_instance = new ContractPromise(api, vault_data.abi, vault.address)

  console.log('Fetching registry contract ...')
  const registry_contract_result = await contractQuery(
    api,
    '',
    vault_instance,
    'get_registry_contract',
  )
  registry.address = decodeOutput(registry_contract_result, vault_instance, 'get_registry_contract').output
  const registry_instance = new ContractPromise(api, registry_data.abi, registry.address)
  console.log(`Registry Address: ${registry.address}`)

  console.log('Fetching share token contract ...')
  const share_token_contract_result = await contractQuery(
    api,
    '',
    vault_instance,
    'get_share_token_contract',
  )
  share_token.address = decodeOutput(share_token_contract_result, vault_instance, 'get_share_token_contract').output
  console.log(`Share Token Address: ${share_token.address}`)

  for (let i=0; i<nomination_agents.length; i++) {
    console.log(`Adding nomination agent (${i+1}/${nomination_agents.length}) to registry ...`)
    await contractTx(
      api,
      account,
      registry_instance,
      'add_agent',
      {},
      [nomination_agents[i].address, '1000'],
    )
    console.log('Success!')
  }

  await writeContractAddresses(chain.network, {
    vault,
    share_token,
    registry,
  })
}

const POOL_IDS = [1, 2]

main(POOL_IDS)
  .catch((error) => {
    console.error(error)
    process.exit(1)
  })
  .finally(() => process.exit(0))
