-- ========================================================
-- 0. Helper Functions
-- ========================================================

-- Function to ensure category is not top-level (parent_id IS NULL)
CREATE OR REPLACE FUNCTION public.verify_product_category_is_leaf()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM public.categories 
        WHERE id = NEW.category_id 
          AND parent_id IS NULL
    ) THEN
        RAISE EXCEPTION 'Category ID % is a top-level category and cannot be assigned to products.', NEW.category_id
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ========================================================
-- 1. Table: public.products
-- ========================================================

-- DROP TABLE IF EXISTS public.products CASCADE;

CREATE TABLE IF NOT EXISTS public.products
(
    id bigint NOT NULL GENERATED ALWAYS AS IDENTITY,
    category_id bigint NOT NULL,
    name_en character varying(255) COLLATE pg_catalog."default" NOT NULL,
    name_ar character varying(255) COLLATE pg_catalog."default" NOT NULL,
    notes text COLLATE pg_catalog."default",
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now(),

    CONSTRAINT products_pkey PRIMARY KEY (id),
    CONSTRAINT fk_products_category FOREIGN KEY (category_id)
        REFERENCES public.categories (id)
        ON DELETE RESTRICT
)
TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.products OWNER to mohammed;

-- ========================================================
-- 2. Indexes for products
-- ========================================================

CREATE UNIQUE INDEX IF NOT EXISTS idx_products_unique_name_en_lower
    ON public.products USING btree
    (lower(name_en::text) COLLATE pg_catalog."default" ASC NULLS LAST)
    WITH (fillfactor=100, deduplicate_items=True)
    TABLESPACE pg_default;

CREATE UNIQUE INDEX IF NOT EXISTS idx_products_unique_name_ar_lower
    ON public.products USING btree
    (lower(name_ar::text) COLLATE pg_catalog."default" ASC NULLS LAST)
    WITH (fillfactor=100, deduplicate_items=True)
    TABLESPACE pg_default;

CREATE INDEX IF NOT EXISTS idx_products_category_id 
    ON public.products USING btree (category_id)
    TABLESPACE pg_default;

-- ========================================================
-- 3. Triggers for products
-- ========================================================

-- Trigger 1: Automatic updated_at timestamp maintenance
DROP TRIGGER IF EXISTS update_products_modtime ON public.products;
CREATE TRIGGER update_products_modtime
    BEFORE UPDATE ON public.products
    FOR EACH ROW
    EXECUTE FUNCTION public.update_modified_column();

-- Trigger 2: Category level verification before insert or update
DROP TRIGGER IF EXISTS check_product_category_before_save ON public.products;
CREATE TRIGGER check_product_category_before_save
    BEFORE INSERT OR UPDATE OF category_id ON public.products
    FOR EACH ROW
    EXECUTE FUNCTION public.verify_product_category_is_leaf();