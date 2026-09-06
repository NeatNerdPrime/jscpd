import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.RUNTIME)
public @interface Tag {
    String value() default "";
    int priority() default 0;
    boolean inherited() default false;
    String[] tags() default {};
}
