# EduChain: Blockchain-Powered Educational Platform

## Overview

EduChain is an innovative blockchain-based educational platform designed to revolutionize learning by creating a transparent, rewarding, and engaging ecosystem for students, teachers, and content creators.

## Key Features

### 1. Participant Roles
- **Students**: Learn and earn rewards based on their educational achievements
- **Teachers**: 
  - Enroll students on the platform
  - Create and manage educational content
  - Earn fees from student interactions
- **Content Creators**: Develop high-quality educational materials

### 2. Reward Mechanism
- **Learn to Earn**: Students are incentivized through a reward system that recognizes and compensates educational progress
- Teachers may receive a percentage of student spending on platform features

### 3. Platform Functionality
- Paid extra features including:
  - New learning tasks
  - Detailed solutions
  - Unit tests and assessments
- NFT-based certification system
- Transparent transaction tracking

## Technical Architecture

### Smart Contract Capabilities
- NFT token issuance for certificates and user registration
  - Each teachers will have an NFT collection and will create NFTs for the students they enroll. This will be done via `registerUser` endpoint;
    - [Here](https://devnet-explorer.multiversx.com/transactions/36b3f281a159d27962f0ea33b8a460bc94002feba54b2b7e52c7defc120108f1) is an example of creating an NFT collection via calling a SC;
    - [Here](https://devnet-explorer.multiversx.com/transactions/a8a9b89c5fe02251be60ee8256e84acd126d8d0d489476546909a8c1e376c131) is an example for registering a user;
  - After graduating a course, a certificate can be issued to a student. The certificate is an NFT with specific attributes: Name, Program Name, Grade, Credit Points, Expiration Date, etc.);
    - [Here](https://devnet-explorer.multiversx.com/transactions/b16ba5ccfdf9324526c588cc066a836bf93bd306fb7f30c937cb512605104f24) is an example for issuing a certificate;
- Reward token management
- Owner-controlled administrative functions

### Key Smart Contract Methods
- `issueCertificate()`: Create educational certificates as NFTs
- `registerUser()`: Register new users on the platform
- `depositRewards()`: Add rewards to the platform's reward pool

## Blockchain Details
- Developed for the MultiversX blockchain
- Uses ESDT (Encrypted Standard Digital Token) for transactions
- Supports non-fungible token (NFT) creation

## Getting Started

### Prerequisites
- MultiversX wallet
- Basic understanding of blockchain technologies
- Development environment for MultiversX smart contracts

### Installation
1. Clone the repository
2. Set up MultiversX development tools
3. Deploy the smart contract
4. Issue platform tokens
5. Set local roles for token management

## Future Roadmap
- Enhanced learning analytics
- More granular reward mechanisms
- Expanded content creation tools
- Cross-platform integrations