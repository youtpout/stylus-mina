const { ethers } = require('ethers');

// Configuration
const PRIVATE_KEY = '0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659'; // Replace with your private key
const RPC_URL = 'http://localhost:8547'; // Stylus testnet RPC
const CONTRACT_ADDRESS = '0xc2c0c3398915a2d2e9c33c186abfef3192ee25e8'; // Replace with your deployed contract address

// Contract ABI
const ABI = [
    "function hash(uint256[] memory vec) external view returns (uint256)",
    "function hashFunction(uint256[] memory vec) external returns (uint256)"
];

async function main() {
    try {
        // Connect to the provider
        const provider = new ethers.JsonRpcProvider(RPC_URL);

        // Create wallet instance
        const wallet = new ethers.Wallet(PRIVATE_KEY, provider);

        console.log('🔗 Connecting to network...');
        console.log('Wallet address:', wallet.address);

        // Check wallet balance
        const balance = await provider.getBalance(wallet.address);
        console.log('Balance:', ethers.formatEther(balance), 'ETH');

        // Create contract instance
        const contract = new ethers.Contract(CONTRACT_ADDRESS, ABI, wallet);

        // Test data - array of integers for hashing
        const testVector = [
            ethers.getBigInt("123456789"),
            ethers.getBigInt("987654321"),
            ethers.getBigInt("555666777")
        ];

        console.log('\n📊 Input data:', testVector.map(v => v.toString()));

        // ===== TEST VIEW FUNCTION: hash() =====
        console.log('\n🔍 Calling view function hash()...');

        try {
            // Call the view function
            const viewResult = await contract.hash(testVector);
            console.log('✅ Result from hash():', viewResult.toString());

            // View functions don't cost gas when called directly
            console.log('💰 Actual cost: 0 ETH (view function)');

        } catch (error) {
            console.log('❌ Error calling hash():', error.message);
        }

        // ===== TEST STATE-CHANGING FUNCTION: hashFunction() =====
        console.log('\n🔄 Calling state-changing function hashFunction()...');

        try {
            // Gas estimation for state-changing function
            const txGasEstimate = await contract.hashFunction.estimateGas(testVector);
            console.log('Estimated gas for hashFunction():', txGasEstimate.toString());

            // Get current gas price
            const feeData = await provider.getFeeData();
            console.log('Gas price:', ethers.formatUnits(feeData.gasPrice, 'gwei'), 'gwei');

            // Calculate estimated cost
            const estimatedCost = txGasEstimate * feeData.gasPrice;
            console.log('💰 Estimated cost:', ethers.formatEther(estimatedCost), 'ETH');

            // Execute the transaction
            console.log('⏳ Sending transaction...');
            const tx = await contract.hashFunction(testVector, {
                gasLimit: txGasEstimate + ethers.getBigInt(10000) // Safety margin
            });

            console.log('📤 Transaction hash:', tx.hash);
            console.log('⏳ Waiting for confirmation...');

            // Wait for transaction confirmation
            const receipt = await tx.wait();
            console.log('✅ Transaction confirmed!');
            console.log('Block:', receipt.blockNumber);
            console.log('Gas used:', receipt.gasUsed.toString());

            // Calculate actual cost
            const actualCost = receipt.gasUsed * receipt.gasPrice;
            console.log('💰 Actual cost:', ethers.formatEther(actualCost), 'ETH');

            // Get result from logs or separate call
            console.log('🔍 Fetching result...');
            const result = await contract.hash(testVector); // Use view function to get result
            console.log('✅ Result from hashFunction():', result.toString());

        } catch (error) {
            console.log('❌ Error calling hashFunction():', error.message);

            // Display gas info if available in error
            if (error.reason) {
                console.log('Reason:', error.reason);
            }
        }

    } catch (error) {
        console.error('❌ General error:', error);
    }
}

// Utility function to compare gas costs across different vector sizes
async function compareGasCosts() {
    console.log('\n📈 GAS COST COMPARISON');
    console.log('='.repeat(50));

    try {
        const provider = new ethers.JsonRpcProvider(RPC_URL);
        const wallet = new ethers.Wallet(PRIVATE_KEY, provider);
        const contract = new ethers.Contract(CONTRACT_ADDRESS, ABI, wallet);

        // Test with different vector sizes
        const sizes = [1, 2, 5, 10];

        for (const size of sizes) {
            // Create vector with specified size
            const vector = Array(size).fill().map((_, i) => ethers.getBigInt(i + 1));

            try {
                const gasEstimate = await contract.hashFunction.estimateGas(vector);
                console.log(`Size ${size}: ${gasEstimate.toString()} gas`);
            } catch (error) {
                console.log(`Size ${size}: Error - ${error.message}`);
            }
        }

    } catch (error) {
        console.log('Error during comparison:', error.message);
    }
}

// Script execution
console.log('🚀 Starting Poseidon Contract Script');
console.log('='.repeat(50));

main()
    .then(() => compareGasCosts())
    .then(() => {
        console.log('\n✅ Script completed successfully!');
        process.exit(0);
    })
    .catch((error) => {
        console.error('💥 Fatal error:', error);
        process.exit(1);
    });