CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Utilisateurs de la plateforme
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sessions révocables (Stateful Token-based avec Cookies HttpOnly)
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) UNIQUE NOT NULL,
    user_agent TEXT,
    ip_address INET,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_token_hash ON user_sessions(token_hash);

-- Registre des modules (Architecture modulaire)
CREATE TABLE available_modules (
    slug VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT
);

INSERT INTO available_modules (slug, name, description) VALUES
    ('habits_health', 'Habitudes & Santé', 'Suivi routines, entraînements et métriques'),
    ('education', 'Éducation & Cours', 'Gestion des cours, coefficients et notes'),
    ('projects_roadmap', 'Projets & Roadmap', 'Arbre de compétences et suivi Git'),
    ('calendar_org', 'Organisation & Agenda', 'Emploi du temps consolidé et événements'),
    ('atelier_recipes', 'Atelier & Recettes', 'Photos créations manuelles et carnet de recettes'),
    ('finances_advisor', 'Finances & IA', 'Budget, portefeuille et agent de veille');

-- Modules activés par utilisateur
CREATE TABLE user_modules (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    module_slug VARCHAR(50) NOT NULL REFERENCES available_modules(slug) ON DELETE CASCADE,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, module_slug)
);