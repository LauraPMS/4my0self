ALTER TABLE user_modules ENABLE ROW LEVEL SECURITY;

-- Politique d'isolation : chaque utilisateur ne peut requêter que ses propres modules
CREATE POLICY user_modules_isolation ON user_modules
    FOR ALL
    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);